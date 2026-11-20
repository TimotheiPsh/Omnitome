# Phase 2D: Rust Extraction Supervisor Security Review

> **Authoritative Security Review & Vulnerability Assessment**  
> **Status:** Review Completed / Remediation Plan Formulated  
> **Target Subsystem:** Tauri Desktop Shell Child-Process Supervisor (`src-tauri/src/lib.rs`)  
> **Scope:** `unpack_managed_archive_7z`, `ExtractionLock`, `validate_archive_entries`, `validate_entry_name`, `find_node_executable`  
> **Author:** Shiori & Onii-sama  
> **Verdict:** **CONDITIONAL GO** (Architectural foundation verified; high-severity edge cases in directory promotion, stale lock race, and NTFS alternate data streams require hardening before commit)

---

## 1. Executive Summary & Assessment Scope

Following the successful compression measurements in Phase 2D (saving 263.57 MB and establishing ~190 MB headroom below the 32-bit NSIS 2.047 GB boundary), this focused security review evaluates the newly introduced Rust archive extraction supervisor in [src-tauri/src/lib.rs](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/src-tauri/src/lib.rs).

The supervisor is responsible for safely extracting compressed archives (`node.7z`, `backend.7z`, and `sidecar.7z`) from application resources into `%LOCALAPPDATA%\Omnitome\`. Because this code operates before user authentication, with elevated process trust, and manages executable runtimes (`node.exe`, `server.js`, `omnitome-sidecar.exe`), it must adhere to strict security invariants:
1. **Integrity Invariant:** Fast-path skips must never accept corrupted, truncated, or incomplete runtime installations, nor bypass version upgrades.
2. **Concurrency Invariant:** Simultaneous multi-process launches must serialize cleanly without deadlocks, corrupted extractions, or stale-lock deletion races.
3. **Atomicity Invariant:** Decompression and directory promotion must be transactional; a failure during extraction or promotion must **never** destroy an existing valid runtime.
4. **Extraction Containment Invariant:** The archive extractor must be resilient against ZipSlip, path traversal, Windows reserved names, NTFS Alternate Data Streams (ADS), symlinks, and decompression bombs.

---

## 2. Technical Findings by Domain

### 2.1 Extraction Completion Verification & Fast-Path Integrity
*Code Reference:* `src-tauri/src/lib.rs`, lines 392–397, 404–407, 717–720.

```rust
// Fast-path: if target directory already exists and contains the expected verification file, return immediately
let expected_path = target_dir.join(expected_file);
if target_dir.exists() && (expected_path.exists() || target_dir.join("omnitome-sidecar").join(expected_file).exists()) {
    return Ok(());
}
```

#### Detailed Findings:
1. **Shallow Single-File Existence Check:**
   - The fast-path relies exclusively on `expected_path.exists()`.
   - On Windows, `exists()` returns `true` even for a 0-byte stub, an empty file, or a corrupted partial binary.
   - For `node.exe`: A 0-byte stub created by an interrupted disk write or external process satisfies `exists()`.
   - For `backend`: `server.js` is merely 1 file out of **6,655 files** in `backend-runtime`. If extraction was terminated while writing `node_modules/`, but `server.js` was already written, the fast-path accepts the runtime as valid. When Node launches, it immediately crashes with `MODULE_NOT_FOUND` on missing native dependencies (e.g. `better-sqlite3`).
2. **The Update Blind Spot (Stale Runtime Retention):**
   - The fast-path does not inspect the source archive's metadata (size, modification time, or cryptographic hash).
   - If a user updates Omnitome from version 1.1.1 to 1.1.2 with an updated `backend.7z` or `node.7z`, the presence of the older `server.js` or `node.exe` in `%LOCALAPPDATA%\Omnitome\` will cause the supervisor to **skip extraction of the updated archive entirely**. The app will run stale runtime code.
3. **Proposed Completion Manifest Architecture:**
   - Rather than checking single file existence, the supervisor must verify a `.extraction_manifest.json` written inside the target directory as the final atomic operation:
     ```json
     {
       "version": 1,
       "archive_name": "backend.7z",
       "archive_size": 54978241,
       "archive_sha256": "4a7b...",
       "extracted_at": "2026-11-20T02:00:00Z",
       "file_count": 6655,
       "total_bytes": 265198445,
       "critical_files": [
         { "relative_path": "server.js", "min_size": 1000 },
         { "relative_path": "package.json", "min_size": 500 }
       ]
     }
     ```
   - **Fast-Path Verification Logic:**
     1. Does `.extraction_manifest.json` exist?
     2. Does `archive_sha256` (or `archive_size` + `mtime`) match the current archive in `resources/`? If the archive has changed, force re-extraction.
     3. Do all `critical_files` exist and exceed `min_size`?
     4. If valid, skip in 0 ms. If invalid or missing, proceed to extraction.

---

### 2.2 Cross-Process Concurrency & Lock Lifecycle
*Code Reference:* `src-tauri/src/lib.rs`, lines 306–354.

```rust
match fs::OpenOptions::new().write(true).create_new(true).open(&lock_path) {
    Ok(mut file) => { ... writeln!(file, "{}", pid); ... return Ok(Self { lock_path }); }
    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
        if let Ok(content) = fs::read_to_string(&lock_path) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if !is_process_alive(pid) {
                    let _ = fs::remove_file(&lock_path);
                    continue;
                }
            }
        }
        ...
    }
}
```

#### Detailed Findings:
1. **Confirmed Protection (Atomic Acquisition):**
   - `OpenOptions::create_new(true)` compiles to Win32 `CreateFileW` with `CREATE_NEW`.
   - If the file exists, the Windows NTFS kernel returns `ERROR_FILE_EXISTS`. This guarantees that only one process can create the lock file; concurrency collisions during initial creation are mathematically impossible.
2. **Race Condition in Stale Lock Reclamation:**
   - Suppose Process X (PID 1000) crashed during extraction and left `.node.lock`.
   - Process A and Process B launch simultaneously.
   - Both detect that PID 1000 is no longer alive (`!is_process_alive(1000)`).
   - Process A executes `fs::remove_file(&lock_path)`, loops, and successfully acquires the lock with `create_new(true)`, writing PID A (PID 2000).
   - **The Collision:** Process B, which also evaluated PID 1000 as dead, now executes its pending statement: `let _ = fs::remove_file(&lock_path);`!
   - Process B **deletes Process A's newly acquired lock file** while Process A is extracting!
   - Process C launches, finds no lock file, acquires the lock, and starts extracting concurrently with Process A, corrupting the staging files.
3. **Sub-Millisecond Uninitialized Read Window:**
   - Process A creates the file with `create_new(true)`. The file is 0 bytes on disk.
   - Before Process A writes `writeln!(file, "{}", pid)` and flushes, Process B attempts to read `fs::read_to_string(&lock_path)`.
   - Process B reads `""`, and `"".trim().parse::<u32>()` fails. Process B sleeps 200ms without crashing. While non-fatal, this is uncoordinated.
4. **Windows PID Recycling:**
   - On Windows, PIDs are aggressively recycled upon process termination. If PID X terminates and Windows reassigns PID X to an unrelated background system process (e.g. `svchost.exe`), `is_process_alive(X)` returns `true`, forcing Omnitome to stall for the full 60-second timeout.
5. **Windows-Native Locking Solution:**
   - **Persistent File Handle with Exclusive Share Mode:** Instead of closing the lock file handle and relying on text parsing, `ExtractionLock` should hold the open `std::fs::File` handle in memory.
   - On Windows, keeping an open handle created with `create_new(true)` locks the file at the kernel level.
   - If the process terminates abnormally (`taskkill /F`, power failure, BSOD), **the Windows kernel automatically closes the file handle and releases the lock**.
   - Even better: using a Windows named mutex (`CreateMutexW(NULL, FALSE, L"Local\\Omnitome_Extract_Lock_node")`). When a process holding an acquired mutex dies, waiting processes immediately receive `WAIT_ABANDONED`, identifying process death instantly with zero disk I/O, zero PID checks, and zero stale files.

---

### 2.3 Atomic Replacement & Disaster Recovery
*Code Reference:* `src-tauri/src/lib.rs`, lines 441–448, 292–304.

```rust
// Step 5: Atomic swap into target directory
if target_dir.exists() {
    wipe_dir(&target_dir)
        .map_err(|e| format!("Failed to remove existing target directory {:?}: {}", target_dir, e))?;
}
atomic_rename_dir(&staging_dir, &target_dir)
    .map_err(|e| format!("Failed to move unpacked content from {:?} to {:?}: {}", staging_dir, target_dir, e))?;
```

#### Detailed Findings:
1. **CRITICAL DEFECT: Wipe-Before-Promote Window of Irrevocable Loss:**
   - In Step 5, if `target_dir` already exists (e.g. an existing valid runtime during an update or re-extraction), the supervisor **deletes `target_dir` before renaming `staging_dir`**.
   - If `wipe_dir(&target_dir)` succeeds, the working runtime is gone.
   - If `atomic_rename_dir(&staging_dir, &target_dir)` subsequently fails (e.g. Windows Defender holds an open handle to inspect a newly unpacked `.dll`, Windows Search Indexer locks a file, or a filesystem error occurs), the operation fails.
   - **The Consequence:** The last valid runtime installation is permanently destroyed, and the new runtime was not promoted. The application is completely broken and cannot launch until manual reinstallation.
2. **Rollback-Safe 3-Way Atomic Directory Rotation Protocol:**
   - To guarantee zero downtime and 100% disaster recovery:
     ```text
     Step A: Rename target_dir -> target_dir.backup
     Step B: Rename staging_dir -> target_dir
     Step C: If Step B succeeds:
                wipe_dir(target_dir.backup)
             If Step B fails:
                // ROLLBACK! Restore working runtime
                rename target_dir.backup -> target_dir
     ```
   - Under this protocol, an unexpected failure during promotion automatically restores the previous working runtime, maintaining application availability.

---

### 2.4 Archive Extraction Security & Decompression Hardening
*Code Reference:* `src-tauri/src/lib.rs`, lines 356–381.

```rust
fn validate_entry_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if name.contains("..") { ... }
    if name.starts_with('/') || name.starts_with('\\') { ... }
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' { ... }
    Ok(())
}
```

#### Detailed Findings:
1. **Confirmed Protections:**
   - Successfully blocks parent directory traversal (`..`).
   - Successfully blocks absolute Unix paths (`/etc/...`) and absolute Windows paths (`\Windows\...`).
   - Successfully blocks drive specifiers at the root (`C:\...`, `D:/...`).
2. **NTFS Alternate Data Stream (ADS) Bypass:**
   - `bytes[1] == b':'` only checks the second character of the root.
   - An entry named `server.js:hidden_payload` or `sub/module.js::$DATA` does not trigger the drive specifier check and contains no `..`.
   - On NTFS, colons are reserved for Alternate Data Streams. Writing to an ADS can evade security tools or cause filesystem corruption.
   - **Remedy:** Reject any entry where `name.contains(':')`.
3. **Windows DOS Reserved Device Names:**
   - Win32 treats specific legacy names as devices regardless of directory: `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`, `LPT1`–`LPT9` (case-insensitive, with or without extensions like `CON.txt` or `nul.js`).
   - Attempting to create or open a file named `aux.js` or `NUL` can cause the thread to hang indefinitely or fail.
   - `validate_entry_name` does not check for DOS reserved device names.
   - **Remedy:** Extract the file stem (before extension) of each path component; reject if it matches any DOS reserved name.
4. **Decompression Bomb Protection (Unbounded Resource Limits):**
   - `validate_archive_entries` iterates the central directory entries, but imposes no thresholds on:
     - Total uncompressed size (e.g. limit to 2.0 GB).
     - Total file count (e.g. limit to 15,000 files).
     - Maximum individual file size (e.g. limit to 500 MB).
   - A maliciously crafted 10 MB 7z archive could expand to 500 GB, exhausting disk space.
   - **Remedy:** Accumulate `entry.size()` across all entries during `validate_archive_entries`. If total size > 2.0 GB or entry count > 15,000, reject the archive before decompressing.
5. **Symlink and Reparse Point Safety:**
   - 7z archives support symbolic links (`kSymbolicLink`). If an entry creates a symlink pointing to an arbitrary host directory, subsequent extracted files could be written through that link outside the target directory.
   - **Remedy:** Assert that no entry in the archive possesses symlink or reparse point attributes.
6. **Two-Pass TOCTOU (Time-Of-Check to Time-Of-Use):**
   - The archive is inspected in Pass 1 (`validate_archive_entries`), closed, and re-opened in Pass 3 (`sevenz_rust::decompress_file`).
   - While the archive path in `%PROGRAMFILES%\Omnitome\resources\` is read-only for unprivileged users, in developer mode or `%LOCALAPPDATA%` the archive could theoretically be swapped between check and use.
   - **Remedy:** In production, open the archive file with `FILE_SHARE_READ` and maintain the handle or ensure the resource directory permissions are strictly verified.

---

## 3. Vulnerability & Risk Register

| ID | Severity | Category | Description | Practical Impact | Remediation Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **V-01** | **HIGH** | Reliability / Atomicity | **Wipe-before-promote pattern:** `wipe_dir(target_dir)` executes before `atomic_rename_dir(staging, target)`. | If rename fails (e.g. AV scan lock), existing valid runtime is permanently destroyed. | **Fix Required** (Implement 3-way rotation with rollback) |
| **V-02** | **HIGH** | Lifecycle / Security | **Fast-path accepts stale runtimes:** No archive hash, version stamp, or timestamp comparison. | Application updates fail to deploy new archives; stale code executes indefinitely. | **Fix Required** (Implement `.extraction_manifest.json` with archive hash) |
| **V-03** | **HIGH** | Concurrency | **Stale lock reclamation race:** Waiting Process B deletes Process A's newly acquired lock file. | Multiple concurrent processes can decompress simultaneously, corrupting staging files. | **Fix Required** (Hold persistent open file handle / OS share lock) |
| **V-04** | **MEDIUM** | Archive Security | **NTFS Alternate Data Streams (ADS):** Entry names with colons (`:`) not at index 1 are permitted. | Malicious archive can write alternate data streams or trigger unexpected NTFS behavior. | **Fix Required** (Reject any entry containing `:`) |
| **V-05** | **MEDIUM** | Archive Security | **Windows DOS reserved names:** `CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9` unhandled. | Win32 file creation can hang indefinitely or fail with access denied. | **Fix Required** (Filter path segment file stems against DOS device table) |
| **V-06** | **MEDIUM** | Resource Safety | **No decompression bomb guards:** Unchecked total size and file count in archive validation. | Malicious or corrupt archive could exhaust disk space or crash system memory. | **Fix Required** (Enforce max 2GB uncompressed size and max 15,000 files) |
| **V-07** | **MEDIUM** | Integrity | **Fast-path accepts 0-byte stub:** `expected_path.exists()` does not check file length or completeness. | A 0-byte truncated executable satisfies the fast path, inducing startup failures. | **Fix Required** (Enforce non-zero size checks in manifest verification) |
| **V-08** | **LOW** | Concurrency | **Windows PID recycling:** Reused PID could induce a 60-second lock acquisition timeout. | Minor delay on launch after abnormal process crash. | Resolved by persistent file handle / mutex |
| **V-09** | **LOW** | Security | **Two-pass TOCTOU window:** Archive is checked in pass 1 and re-opened in pass 2. | Low likelihood; mitigated by Windows resource directory access controls. | Documented & acceptable for local release |

---

## 4. Confirmed Protections

1. **Path Traversal Containment (`..` and Root Separators):**
   - `validate_entry_name` conclusively catches and rejects all `../`, `..\\`, `/root`, and `\root` entry attacks. Verified by unit tests `test_validate_entry_name_path_traversal` and `test_validate_archive_entries_rejects_malicious`.
2. **Initial Mutual Exclusion:**
   - Initial acquisition via `OpenOptions::create_new(true)` is strictly atomic at the Windows NTFS kernel layer. Two processes racing to create the lock file simultaneously can never both succeed.
3. **Staging Directory Isolation:**
   - Decompression writes exclusively into `<target_dir>.staging`, preventing active runtimes from reading partial files while extraction is in progress.
4. **Pre-Swap File Sanity:**
   - Step 4 asserts that the key executable exists in staging before any directory rename is attempted.
5. **Disk Space Reclamation:**
   - Optional removal of temporary archives (`delete_archive_on_success`) reclaims disk space after verified promotion.

---

## 5. Recommended Remediation Plan (Prioritized)

### Priority 1: High-Severity Fixes (Required Before Commit)

#### Remediation 1.1: Rollback-Safe 3-Way Atomic Directory Rotation
Replace lines 442–448 in `src-tauri/src/lib.rs` with:
```rust
let backup_dir = target_dir.with_extension("backup");
if backup_dir.exists() {
    let _ = wipe_dir(&backup_dir);
}

// Phase 1: Move existing target to backup if it exists
let had_existing_target = target_dir.exists();
if had_existing_target {
    atomic_rename_dir(&target_dir, &backup_dir)
        .map_err(|e| format!("Failed to move existing target to backup: {}", e))?;
}

// Phase 2: Promote staging to target
if let Err(e) = atomic_rename_dir(&staging_dir, &target_dir) {
    eprintln!("[Supervisor] Promotion failed: {}. Attempting rollback...", e);
    if had_existing_target && backup_dir.exists() {
        let _ = atomic_rename_dir(&backup_dir, &target_dir);
    }
    return Err(format!("Failed to promote staging to target (rollback attempted): {}", e).into());
}

// Phase 3: Cleanup backup after confirmed promotion
if backup_dir.exists() {
    let _ = wipe_dir(&backup_dir);
}
```

#### Remediation 1.2: Persistent Lock File Handle (Eliminate Stale Race)
In `ExtractionLock`, hold the `fs::File` open in the struct:
```rust
struct ExtractionLock {
    lock_path: PathBuf,
    _file: fs::File,
}
```
Opening with `create_new(true)` on Windows holds the handle open without delete sharing, completely preventing waiting processes from deleting an active lock. On abnormal process termination, Windows NTFS kernel releases the handle automatically.

#### Remediation 1.3: Completion Manifest with Archive Hash
Generate `.extraction_manifest.json` containing `archive_sha256`, `total_files`, and `extracted_at`.
In the fast path, read `.extraction_manifest.json` and compute/compare the current archive's SHA-256 (or size/mtime). If mismatched, do not skip.

---

### Priority 2: Medium-Severity Fixes

#### Remediation 2.1: Harden `validate_entry_name` (Colons & DOS Devices)
Update `validate_entry_name`:
```rust
fn validate_entry_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if name.contains("..") {
        return Err(format!("Parent traversal rejected: {:?}", name).into());
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return Err(format!("Root separator rejected: {:?}", name).into());
    }
    // Reject any colon (NTFS Alternate Data Streams or drive specifiers)
    if name.contains(':') {
        return Err(format!("Colon / Alternate Data Stream rejected: {:?}", name).into());
    }
    // Check Windows DOS reserved names
    let dos_devices = ["CON", "PRN", "AUX", "NUL",
                       "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
                       "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    for part in Path::new(name).components() {
        let s = part.as_os_str().to_string_lossy();
        let stem = s.split('.').next().unwrap_or("").to_ascii_uppercase();
        if dos_devices.contains(&stem.as_str()) {
            return Err(format!("Windows DOS reserved name rejected: {:?}", name).into());
        }
    }
    Ok(())
}
```

#### Remediation 2.2: Decompression Bomb Safeguards in `validate_archive_entries`
Accumulate entry counts and uncompressed byte sizes in `validate_archive_entries`:
- Max entries: `25,000`
- Max total uncompressed bytes: `3.0 GB`
- Reject immediately if exceeded.

---

## 6. Authoritative Review Verdict

### **Verdict: CONDITIONAL GO**

1. **Why Not NO-GO?**
   - The fundamental architectural direction (Path A: 7z archive compression with Rust supervisor extraction) is completely validated.
   - The NSIS payload reduction (saving 263.57 MB, securing 190 MB headroom) solves the packaging ceiling cleanly without removing CUDA/cuDNN or downgrading PyTorch.
   - Path traversal prevention, process supervisor lifecycles, and basic atomic staging are already operational.
2. **Why Conditional?**
   - Committing the code in its current prototype form would introduce a high-severity reliability risk (V-01: potential destruction of valid runtimes on rename failure) and an update blind spot (V-02: fast-path skipping version updates).
   - Addressing Remediation 1.1 (3-way rotation), Remediation 1.2 (persistent handle locking), Remediation 1.3 (manifest verification), and Remediation 2.1 (colon & DOS device filtering) is straightforward and will make the Rust supervisor enterprise-grade.

---

## 7. Next Actions Awaiting User Decision

1. **Review Findings:** Await Onii-sama's confirmation of the review findings in this document.
2. **Authorize Hardening Implementation:** Upon authorization, implement Remediations 1.1, 1.2, 1.3, 2.1, and 2.2 in `src-tauri/src/lib.rs`.
3. **Execute Verification Suites:** Re-run `cargo test --manifest-path src-tauri/Cargo.toml` and full regression suites to confirm all gates pass.
4. **Final Commit Authorization:** Stage and commit the hardened Rust supervisor.
