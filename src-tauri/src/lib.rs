use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::Manager;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Serialize, Deserialize, Default, Debug)]
struct PidsFile {
    sidecar_pid: Option<u32>,
    node_pid: Option<u32>,
}

struct ProcessManager {
    sidecar: Mutex<Option<Child>>,
    node: Mutex<Option<Child>>,
    sidecar_pid: u32,
    node_pid: u32,
    app_dir: PathBuf,
}

#[derive(Clone, Copy, Debug)]
pub struct DynamicPorts {
    pub node_port: u16,
    pub sidecar_port: u16,
}

impl ProcessManager {
    fn shutdown(&self) {
        // Forcefully terminate Python sidecar and all sub-processes (PaddleOCR, Uvicorn, etc.)
        kill_pid(self.sidecar_pid);
        if let Ok(mut lock) = self.sidecar.lock() {
            if let Some(mut child) = lock.take() {
                kill_pid(child.id());
                let _ = child.kill();
            }
        }
        // Forcefully terminate Node backend and any child processes
        kill_pid(self.node_pid);
        if let Ok(mut lock) = self.node.lock() {
            if let Some(mut child) = lock.take() {
                kill_pid(child.id());
                let _ = child.kill();
            }
        }
        // Sweep any recorded PIDs from disk
        cleanup_stale_pids(&self.app_dir);
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn strip_windows_verbatim(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{}", stripped))
    } else if let Some(stripped) = s.strip_prefix(r"\\?\unc\") {
        PathBuf::from(format!(r"\\{}", stripped))
    } else if let Some(stripped) = s.strip_prefix(r"//?/UNC/") {
        PathBuf::from(format!(r"//{}", stripped))
    } else if let Some(stripped) = s.strip_prefix(r"//?/unc/") {
        PathBuf::from(format!(r"//{}", stripped))
    } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else if let Some(stripped) = s.strip_prefix(r"\\?/") {
        PathBuf::from(stripped)
    } else if let Some(stripped) = s.strip_prefix(r"//?/") {
        PathBuf::from(stripped)
    } else if let Some(stripped) = s.strip_prefix(r"//?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

fn get_app_data_dir() -> PathBuf {
    let local = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into()));
    let base = strip_windows_verbatim(&PathBuf::from(local)).join("Omnitome");
    let _ = fs::create_dir_all(base.join("logs"));
    let _ = fs::create_dir_all(base.join("models"));
    strip_windows_verbatim(&base)
}

fn log_crash(error_msg: &str) {
    let app_dir = get_app_data_dir();
    let crash_log = app_dir.join("logs").join("crash.log");
    rotate_log_if_needed(&crash_log);
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let entry = format!("[{}] [CRASH] {}\n", ts, error_msg);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&crash_log) {
        let _ = f.write_all(entry.as_bytes());
        let _ = f.flush();
    }
    eprintln!("[CRASH] {}", error_msg);
}

#[cfg(windows)]
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    let msg_wide: Vec<u16> = OsStr::new(message).encode_wide().chain(Some(0)).collect();

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: *mut std::ffi::c_void, text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    unsafe {
        // MB_OK (0x0) | MB_ICONERROR (0x10) | MB_SYSTEMMODAL (0x1000) | MB_SETFOREGROUND (0x10000) | MB_TOPMOST (0x40000) = 0x51010
        MessageBoxW(std::ptr::null_mut(), msg_wide.as_ptr(), title_wide.as_ptr(), 0x51010);
    }
}

#[cfg(not(windows))]
fn show_error_dialog(title: &str, message: &str) {
    eprintln!("{}: {}", title, message);
}


fn rotate_log_if_needed(log_path: &Path) {
    if let Ok(meta) = fs::metadata(log_path) {
        if meta.len() > 5 * 1024 * 1024 {
            let old_path = log_path.with_extension("old.log");
            let _ = fs::rename(log_path, old_path);
        }
    }
}

fn kill_pid(pid: u32) {
    if pid == 0 {
        return;
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("taskkill");
        cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let _ = cmd.status();
    }
    #[cfg(not(windows))]
    {
        let mut cmd = Command::new("kill");
        cmd.args(["-9", &pid.to_string()]);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        let _ = cmd.status();
    }
}

#[allow(dead_code)]
fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("tasklist");
        cmd.args(["/FI", &format!("PID eq {}", pid), "/NH"]);
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd.output() {
            let s = String::from_utf8_lossy(&output.stdout);
            return s.contains(&pid.to_string());
        }
        false
    }
    #[cfg(not(windows))]
    {
        let res = unsafe { libc::kill(pid as libc::pid_t, 0) };
        res == 0
    }
}

fn cleanup_stale_pids(app_dir: &Path) {
    let pids_path = app_dir.join("pids.json");
    if pids_path.exists() {
        if let Ok(content) = fs::read_to_string(&pids_path) {
            if let Ok(pids) = serde_json::from_str::<PidsFile>(&content) {
                if let Some(pid) = pids.sidecar_pid {
                    kill_pid(pid);
                }
                if let Some(pid) = pids.node_pid {
                    kill_pid(pid);
                }
            }
        }
        let _ = fs::remove_file(&pids_path);
    }
}

fn write_atomic_pids(app_dir: &Path, sidecar_pid: u32, node_pid: u32) {
    let pids = PidsFile {
        sidecar_pid: Some(sidecar_pid),
        node_pid: Some(node_pid),
    };
    let tmp_path = app_dir.join("pids.json.tmp");
    let final_path = app_dir.join("pids.json");
    if let Ok(json) = serde_json::to_string_pretty(&pids) {
        if fs::write(&tmp_path, json).is_ok() {
            let _ = fs::rename(tmp_path, final_path);
        }
    }
}

#[cfg(debug_assertions)]
fn find_project_root() -> PathBuf {
    if let Ok(dir) = std::env::current_dir() {
        if dir.join("backend").join("server.js").exists() {
            return dir;
        }
        if dir.join("..").join("backend").join("server.js").exists() {
            return dir.join("..").canonicalize().unwrap_or_else(|_| dir.join(".."));
        }
    }
    if let Ok(mut exe) = std::env::current_exe() {
        while exe.pop() {
            if exe.join("backend").join("server.js").exists() {
                return exe;
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn wipe_dir(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    // Attempt direct removal first
    if fs::remove_dir_all(dir).is_ok() {
        return Ok(());
    }

    // On Windows, read-only file attributes or indexing can cause remove_dir_all to fail.
    // Recursively clear read-only permissions and retry with backoff.
    fn unset_readonly_recursive(p: &Path) {
        if let Ok(entries) = fs::read_dir(p) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(meta) = entry.metadata() {
                    let mut perms = meta.permissions();
                    if perms.readonly() {
                        perms.set_readonly(false);
                        let _ = fs::set_permissions(&path, perms);
                    }
                    if meta.is_dir() {
                        unset_readonly_recursive(&path);
                    }
                }
            }
        }
    }
    unset_readonly_recursive(dir);

    let mut last_err = None;
    for attempt in 0..5 {
        match fs::remove_dir_all(dir) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
                thread::sleep(Duration::from_millis(50 * (attempt + 1)));
            }
        }
    }

    if dir.exists() {
        Err(last_err.unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to remove directory")))
    } else {
        Ok(())
    }
}

fn atomic_rename_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut last_err = None;
    for attempt in 0..5 {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
                thread::sleep(Duration::from_millis(50 * (attempt + 1)));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Failed to rename directory")))
}

// --- SHA-256 Digest Implementation ---

pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn compute_file_sha256(path: &Path) -> std::io::Result<String> {
    let file = fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(128 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// --- Windows-Native Inter-Process Synchronization ---

#[cfg(windows)]
pub mod platform_lock {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::time::Duration;

    type HANDLE = *mut std::ffi::c_void;
    type BOOL = i32;
    type DWORD = u32;

    const WAIT_OBJECT_0: DWORD = 0x00000000;
    const WAIT_ABANDONED: DWORD = 0x00000080;
    const WAIT_TIMEOUT: DWORD = 0x00000102;
    const WAIT_FAILED: DWORD = 0xFFFFFFFF;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(
            lpMutexAttributes: *mut std::ffi::c_void,
            bInitialOwner: BOOL,
            lpName: *const u16,
        ) -> HANDLE;
        fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
        fn ReleaseMutex(hMutex: HANDLE) -> BOOL;
        fn CloseHandle(hObject: HANDLE) -> BOOL;
        fn GetLastError() -> DWORD;
    }

    pub struct OsMutex {
        handle: HANDLE,
    }

    unsafe impl Send for OsMutex {}
    unsafe impl Sync for OsMutex {}

    impl OsMutex {
        pub fn acquire(name: &str, timeout: Duration) -> Result<(Self, bool), String> {
            let name_wide: Vec<u16> = OsStr::new(name).encode_wide().chain(Some(0)).collect();
            let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name_wide.as_ptr()) };
            if handle.is_null() {
                let err = unsafe { GetLastError() };
                return Err(format!("CreateMutexW failed with code {}", err));
            }

            let timeout_ms = if timeout.as_millis() > (u32::MAX as u128) {
                u32::MAX
            } else {
                timeout.as_millis() as u32
            };

            let res = unsafe { WaitForSingleObject(handle, timeout_ms) };
            match res {
                WAIT_OBJECT_0 => Ok((Self { handle }, false)),
                WAIT_ABANDONED => {
                    eprintln!("[Supervisor] Acquired abandoned mutex {:?} (previous owner terminated abnormally)", name);
                    Ok((Self { handle }, true))
                }
                WAIT_TIMEOUT => {
                    unsafe { CloseHandle(handle) };
                    Err(format!("Timeout ({:?}) waiting for mutex {:?}", timeout, name))
                }
                WAIT_FAILED => {
                    let err = unsafe { GetLastError() };
                    unsafe { CloseHandle(handle) };
                    Err(format!("WaitForSingleObject failed on mutex {:?} with code {}", name, err))
                }
                other => {
                    unsafe { CloseHandle(handle) };
                    Err(format!("Unexpected wait result {} on mutex {:?}", other, name))
                }
            }
        }
    }

    impl Drop for OsMutex {
        fn drop(&mut self) {
            unsafe {
                ReleaseMutex(self.handle);
                CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(not(windows))]
pub mod platform_lock {
    use std::time::Duration;
    pub struct OsMutex;
    impl OsMutex {
        pub fn acquire(_name: &str, _timeout: Duration) -> Result<(Self, bool), String> {
            Ok((Self, false))
        }
    }
}

// --- Cross-Process Extraction Lock ---

pub struct ExtractionLock {
    #[cfg(windows)]
    _os_mutex: platform_lock::OsMutex,
    pub lock_path: PathBuf,
    pub was_abandoned: bool,
}

pub fn compute_mutex_name(lock_dir: &Path, name: &str) -> String {
    let clean_path = strip_windows_verbatim(lock_dir);
    let normalized_path = clean_path.to_string_lossy().replace('/', "\\").to_lowercase();
    let path_hash = compute_sha256(normalized_path.as_bytes());
    let safe_name: String = name.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    format!("Local\\Omnitome_Extract_{}_{}", safe_name, &path_hash[..16])
}

impl ExtractionLock {
    pub fn acquire(lock_dir: &Path, name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::acquire_with_timeout(lock_dir, name, Duration::from_secs(60))
    }

    pub fn acquire_with_timeout(lock_dir: &Path, name: &str, timeout: Duration) -> Result<Self, Box<dyn std::error::Error>> {
        let lock_path = lock_dir.join(format!(".{}.lock", name));
        let mutex_name = compute_mutex_name(lock_dir, name);

        let (os_mutex, was_abandoned) = platform_lock::OsMutex::acquire(&mutex_name, timeout)
            .map_err(|e| format!("Failed to acquire extraction lock on {:?}: {}", lock_path, e))?;

        if let Ok(mut file) = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&lock_path)
        {
            let pid = std::process::id();
            let _ = writeln!(file, "{}", pid);
            let _ = file.flush();
        }

        Ok(Self {
            #[cfg(windows)]
            _os_mutex: os_mutex,
            lock_path,
            was_abandoned,
        })
    }
}

impl Drop for ExtractionLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

// --- Decompression Resource Limits ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveLimits {
    pub max_uncompressed_bytes: u64,
    pub max_entries: usize,
    pub max_single_file_bytes: u64,
}

impl ArchiveLimits {
    pub fn node() -> Self {
        Self {
            max_uncompressed_bytes: 200 * 1024 * 1024,      // 200 MB (actual payload: ~83 MB)
            max_entries: 100,                               // 100 entries (actual payload: 1)
            max_single_file_bytes: 150 * 1024 * 1024,       // 150 MB (actual payload: ~83 MB)
        }
    }

    pub fn backend() -> Self {
        Self {
            max_uncompressed_bytes: 800 * 1024 * 1024,      // 800 MB (actual payload: ~253 MB across 7,799 entries)
            max_entries: 20_000,                            // 20,000 entries (actual payload: 7,799)
            max_single_file_bytes: 100 * 1024 * 1024,       // 100 MB (actual largest file: onnxruntime.dll @ 28.8 MB)
        }
    }

    pub fn sidecar() -> Self {
        Self {
            max_uncompressed_bytes: 8 * 1024 * 1024 * 1024, // 8.0 GiB (actual payload: ~5.14 GiB across 10,219 entries)
            max_entries: 25_000,                            // 25,000 entries (actual payload: 10,219)
            max_single_file_bytes: 1536 * 1024 * 1024,      // 1.5 GiB (actual largest file: torch_cuda.dll @ ~957 MB)
        }
    }

    pub fn default_for(target_name: &str, expected_file: &str) -> Self {
        if target_name.eq_ignore_ascii_case("node") || expected_file.eq_ignore_ascii_case("node.exe") {
            Self::node()
        } else if target_name.eq_ignore_ascii_case("backend") || expected_file.eq_ignore_ascii_case("server.js") {
            Self::backend()
        } else if target_name.eq_ignore_ascii_case("sidecar") || expected_file.eq_ignore_ascii_case("omnitome-sidecar.exe") {
            Self::sidecar()
        } else {
            Self {
                max_uncompressed_bytes: 8 * 1024 * 1024 * 1024,
                max_entries: 25_000,
                max_single_file_bytes: 1536 * 1024 * 1024,
            }
        }
    }
}

// --- Windows Path Hardening & Attribute Validation ---

fn is_dos_reserved_device(name: &str) -> bool {
    const RESERVED: &[&str] = &[
        "CON", "PRN", "AUX", "NUL", "CLOCK$",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    RESERVED.iter().any(|&r| r.eq_ignore_ascii_case(name))
}

pub fn validate_entry_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if name.is_empty() {
        // Empty entry name represents the archive root directory in sevenz-rust
        return Ok(());
    }
    // Reject ASCII control characters (0x00..=0x1F)
    if name.chars().any(|c| (c as u32) < 32) {
        return Err("Archive entry contains forbidden null or control character".into());
    }
    // Reject colons anywhere in path (NTFS Alternate Data Streams, drive specifiers, ADS payload evasion)
    if name.contains(':') {
        return Err(format!("Archive entry contains colon / NTFS Alternate Data Stream: {:?}", name).into());
    }
    // Reject Windows forbidden characters: < > " | ? *
    const FORBIDDEN_CHARS: &[char] = &['<', '>', '"', '|', '?', '*'];
    if let Some(bad_char) = name.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
        return Err(format!("Archive entry contains forbidden Windows character {:?}: {:?}", bad_char, name).into());
    }
    if name.contains("..") {
        return Err(format!("Archive entry contains dangerous '..' path traversal: {:?}", name).into());
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return Err(format!("Archive entry contains absolute root path: {:?}", name).into());
    }

    let components: Vec<&str> = name.split(['/', '\\']).collect();
    let num_components = components.len();

    for (idx, &comp) in components.iter().enumerate() {
        if comp.is_empty() {
            // Standard directory entries in 7z may have a trailing slash (e.g. "dir/")
            if idx == num_components - 1 && num_components > 1 {
                continue;
            }
            return Err(format!("Archive entry contains redundant slashes or empty component: {:?}", name).into());
        }

        if comp == "." {
            return Err(format!("Archive entry contains '.' current-directory token: {:?}", name).into());
        }

        if comp.ends_with('.') || comp.ends_with(' ') {
            return Err(format!("Archive entry component ends with dot or space: {:?}", name).into());
        }

        if comp.starts_with(' ') {
            return Err(format!("Archive entry component starts with space: {:?}", name).into());
        }

        let stem = comp.split('.').next().unwrap_or("").trim();
        if is_dos_reserved_device(stem) {
            return Err(format!("Archive entry component is a Windows DOS reserved device name: {:?}", name).into());
        }
    }

    Ok(())
}

pub fn validate_entry_attributes(entry: &sevenz_rust::SevenZArchiveEntry) -> Result<(), Box<dyn std::error::Error>> {
    if entry.has_windows_attributes {
        let attrs = entry.windows_attributes;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x00000400;
        if (attrs & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
            return Err(format!(
                "Archive entry {:?} rejected: reparse point / symlink attribute detected ({:#010x})",
                entry.name(), attrs
            ).into());
        }
        let posix_type = (attrs >> 16) & 0xF000;
        if posix_type == 0xA000 {
            return Err(format!(
                "Archive entry {:?} rejected: POSIX symlink detected (mode {:#06o})",
                entry.name(), (attrs >> 16) & 0xFFFF
            ).into());
        }
    }
    Ok(())
}

pub fn validate_archive_entries(archive_path: &Path, limits: ArchiveLimits) -> Result<(), Box<dyn std::error::Error>> {
    let sz = sevenz_rust::SevenZReader::open(archive_path, sevenz_rust::Password::empty())
        .map_err(|e| format!("Failed to read 7z central directory of {:?}: {}", archive_path, e))?;

    let files = &sz.archive().files;
    if files.len() > limits.max_entries {
        return Err(format!(
            "Archive {:?} contains {} entries, exceeding maximum permitted limit of {}",
            archive_path, files.len(), limits.max_entries
        ).into());
    }

    let mut total_uncompressed_bytes: u64 = 0;
    for entry in files.iter() {
        validate_entry_name(entry.name())?;
        validate_entry_attributes(entry)?;

        if entry.size() > limits.max_single_file_bytes {
            return Err(format!(
                "Archive entry {:?} has uncompressed size {} bytes, exceeding maximum single file limit of {} bytes",
                entry.name(), entry.size(), limits.max_single_file_bytes
            ).into());
        }

        total_uncompressed_bytes = total_uncompressed_bytes.checked_add(entry.size())
            .ok_or_else(|| format!("Integer overflow calculating uncompressed size of {:?}", archive_path))?;

        if total_uncompressed_bytes > limits.max_uncompressed_bytes {
            return Err(format!(
                "Archive {:?} total uncompressed size exceeds maximum permitted limit of {} bytes",
                archive_path, limits.max_uncompressed_bytes
            ).into());
        }
    }

    Ok(())
}

// --- Completion Manifest & Verification ---

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CriticalFileCheck {
    pub relative_path: String,
    pub min_size: u64,
    pub is_directory: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtractionManifest {
    pub version: u32,
    pub archive_name: String,
    pub archive_size: u64,
    pub archive_sha256: String,
    pub extracted_at: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub critical_files: Vec<CriticalFileCheck>,
}

fn collect_critical_files_in_staging(staging_dir: &Path, expected_file: &str) -> Vec<CriticalFileCheck> {
    let mut critical = Vec::new();
    // 1. Always record expected_file with min_size: 1 (non-zero guard)
    critical.push(CriticalFileCheck {
        relative_path: expected_file.to_string(),
        min_size: 1,
        is_directory: false,
    });

    // 2. Discover standard structural markers if present in payload
    if staging_dir.join("package.json").exists() {
        critical.push(CriticalFileCheck {
            relative_path: "package.json".to_string(),
            min_size: 1,
            is_directory: false,
        });
    }
    if staging_dir.join("node_modules").is_dir() {
        critical.push(CriticalFileCheck {
            relative_path: "node_modules".to_string(),
            min_size: 0,
            is_directory: true,
        });
    }
    if staging_dir.join("_internal").is_dir() || staging_dir.join("omnitome-sidecar").join("_internal").is_dir() {
        critical.push(CriticalFileCheck {
            relative_path: "_internal".to_string(),
            min_size: 0,
            is_directory: true,
        });
    }
    if staging_dir.join("config.json").exists() {
        critical.push(CriticalFileCheck {
            relative_path: "config.json".to_string(),
            min_size: 1,
            is_directory: false,
        });
    }

    critical
}

pub fn is_manifest_valid(target_dir: &Path, archive_path: &Path, expected_file: &str) -> bool {
    let manifest_path = target_dir.join(".extraction_manifest.json");
    if !manifest_path.exists() {
        return false;
    }

    let content = match fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let manifest: ExtractionManifest = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return false,
    };

    if manifest.version != 1 {
        return false;
    }

    if archive_path.exists() {
        if let Some(fname) = archive_path.file_name() {
            if !fname.to_string_lossy().eq_ignore_ascii_case(&manifest.archive_name) {
                eprintln!(
                    "[Supervisor] Archive name mismatch for {:?}: manifest={}, current={}. Update detected!",
                    archive_path, manifest.archive_name, fname.to_string_lossy()
                );
                return false;
            }
        }

        if let Ok(meta) = archive_path.metadata() {
            if meta.len() != manifest.archive_size {
                eprintln!(
                    "[Supervisor] Archive size mismatch for {:?}: manifest={}, current={}. Update detected!",
                    archive_path, manifest.archive_size, meta.len()
                );
                return false;
            }
        } else {
            return false;
        }

        match compute_file_sha256(archive_path) {
            Ok(hash) => {
                if !hash.eq_ignore_ascii_case(&manifest.archive_sha256) {
                    eprintln!(
                        "[Supervisor] Archive SHA-256 mismatch for {:?}: manifest={}, current={}. Update detected!",
                        archive_path, manifest.archive_sha256, hash
                    );
                    return false;
                }
            }
            Err(e) => {
                eprintln!("[Supervisor] Failed to compute archive SHA-256 for {:?}: {}", archive_path, e);
                return false;
            }
        }
    }

    // Verify critical files
    for check in &manifest.critical_files {
        let p1 = target_dir.join(&check.relative_path);
        let p2 = target_dir.join("omnitome-sidecar").join(&check.relative_path);
        let path = if p1.exists() {
            Some(p1)
        } else if p2.exists() {
            Some(p2)
        } else {
            None
        };

        match path {
            None => {
                eprintln!("[Supervisor] Manifest check failed: missing critical file {:?}", check.relative_path);
                return false;
            }
            Some(ref p) => {
                if check.is_directory {
                    if !p.is_dir() {
                        eprintln!("[Supervisor] Manifest check failed: {:?} is not a directory", p);
                        return false;
                    }
                } else {
                    let size = p.metadata().map(|m| m.len()).unwrap_or(0);
                    if size < check.min_size {
                        eprintln!(
                            "[Supervisor] Manifest check failed: {:?} size {} < min_size {}",
                            p, size, check.min_size
                        );
                        return false;
                    }
                }
            }
        }
    }

    // Verify expected verification file is non-zero
    let exp1 = target_dir.join(expected_file);
    let exp2 = target_dir.join("omnitome-sidecar").join(expected_file);
    let exp_size = exp1.metadata().map(|m| m.len())
        .or_else(|_| exp2.metadata().map(|m| m.len()))
        .unwrap_or(0);
    if exp_size == 0 {
        eprintln!("[Supervisor] Manifest check failed: expected file {:?} is 0 bytes or missing", expected_file);
        return false;
    }

    true
}

fn write_manifest(
    staging_dir: &Path,
    archive_path: &Path,
    file_count: usize,
    total_bytes: u64,
    critical_files: Vec<CriticalFileCheck>,
) -> Result<(), Box<dyn std::error::Error>> {
    let archive_name = archive_path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let archive_size = fs::metadata(archive_path)?.len();
    let archive_sha256 = compute_file_sha256(archive_path)?;
    let extracted_at = chrono::Utc::now().to_rfc3339();

    let manifest = ExtractionManifest {
        version: 1,
        archive_name,
        archive_size,
        archive_sha256,
        extracted_at,
        file_count,
        total_bytes,
        critical_files,
    };

    let json = serde_json::to_string_pretty(&manifest)?;
    let manifest_path = staging_dir.join(".extraction_manifest.json");
    fs::write(manifest_path, json)?;
    Ok(())
}

// --- Rollback-Safe Promotion & Startup Recovery ---

pub fn recover_interrupted_rotations(target_dir: &Path, expected_file: &str) {
    let backup_dir = target_dir.with_extension("backup");
    let staging_dir = target_dir.with_extension("staging");

    let target_has_expected = target_dir.exists() && (
        target_dir.join(expected_file).exists() || target_dir.join("omnitome-sidecar").join(expected_file).exists()
    );
    let backup_has_expected = backup_dir.exists() && (
        backup_dir.join(expected_file).exists() || backup_dir.join("omnitome-sidecar").join(expected_file).exists()
    );

    // Case 1: Target is missing or broken (e.g. empty stub or interrupted promotion),
    // but backup contains the valid runtime.
    // Invariant: Never delete the last validated runtime -> restore backup -> target.
    if !target_has_expected && backup_has_expected {
        eprintln!(
            "[Supervisor] Interrupted rotation detected: target {:?} missing or broken, but valid backup found. Restoring {:?} -> {:?}",
            target_dir, backup_dir, target_dir
        );
        if target_dir.exists() {
            let _ = wipe_dir(target_dir);
        }
        if let Err(e) = atomic_rename_dir(&backup_dir, target_dir) {
            eprintln!("[Supervisor] Error restoring backup directory to target: {}", e);
        }
        if staging_dir.exists() {
            let _ = wipe_dir(&staging_dir);
        }
        return;
    }

    // Case 2: Target contains the valid runtime and backup_dir exists (promotion finished, but backup cleanup was interrupted).
    if target_has_expected && backup_dir.exists() {
        eprintln!("[Supervisor] Interrupted cleanup detected: removing leftover backup directory {:?}", backup_dir);
        if let Err(e) = wipe_dir(&backup_dir) {
            eprintln!("[Supervisor] Notice: Leftover backup directory cleanup deferred: {}", e);
        }
    }
}

pub fn promote_staging_dir(staging_dir: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let backup_dir = target_dir.with_extension("backup");

    if backup_dir.exists() {
        let _ = wipe_dir(&backup_dir);
        if backup_dir.exists() {
            return Err(format!("Cannot promote: failed to remove stale backup directory {:?}", backup_dir).into());
        }
    }

    let had_existing_target = target_dir.exists();

    // Step A: Move existing target to backup if it exists
    if had_existing_target {
        atomic_rename_dir(target_dir, &backup_dir)
            .map_err(|e| format!("Failed to move existing target {:?} to backup {:?}: {}", target_dir, backup_dir, e))?;
    }

    // Step B: Promote staging to target
    if let Err(e) = atomic_rename_dir(staging_dir, target_dir) {
        eprintln!("[Supervisor] Promotion failed: {}. Attempting rollback...", e);
        if had_existing_target && backup_dir.exists() {
            match atomic_rename_dir(&backup_dir, target_dir) {
                Ok(()) => eprintln!("[Supervisor] Rollback succeeded: previous target runtime restored."),
                Err(rollback_err) => eprintln!("[Supervisor] FATAL: Rollback failed: {}. Backup preserved at {:?}", rollback_err, backup_dir),
            }
        }
        if staging_dir.exists() {
            let _ = wipe_dir(staging_dir);
        }
        return Err(format!("Failed to promote staging to target (rollback attempted): {}", e).into());
    }

    // Step C: Cleanup backup after successful promotion (failure is non-fatal)
    if backup_dir.exists() {
        if let Err(e) = wipe_dir(&backup_dir) {
            eprintln!("[Supervisor] Notice: Cleanup of backup {:?} deferred: {}", backup_dir, e);
        }
    }

    Ok(())
}

pub fn unpack_managed_archive_7z(
    archive_path: &Path,
    target_dir: &Path,
    expected_file: &str,
    delete_archive_on_success: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = strip_windows_verbatim(archive_path);
    let target_dir = strip_windows_verbatim(target_dir);

    // Startup recovery of any interrupted rotations
    recover_interrupted_rotations(&target_dir, expected_file);

    // Fast-path: check if target directory already has a verified manifest matching archive SHA-256
    if is_manifest_valid(&target_dir, &archive_path, expected_file) {
        return Ok(());
    }

    let parent_dir = target_dir.parent().ok_or("Target directory has no parent")?;
    fs::create_dir_all(parent_dir)?;

    let target_name = target_dir.file_name().unwrap_or_default().to_string_lossy();
    let _lock = ExtractionLock::acquire(parent_dir, &target_name)?;

    // Re-check post-lock: another process may have completed extraction while we waited
    recover_interrupted_rotations(&target_dir, expected_file);
    if is_manifest_valid(&target_dir, &archive_path, expected_file) {
        return Ok(());
    }

    eprintln!("[Supervisor] Unpacking managed archive from {:?} to {:?}", archive_path, target_dir);

    let limits = ArchiveLimits::default_for(&target_name, expected_file);

    // Step 1: Pre-extraction archive header and entry validation
    validate_archive_entries(&archive_path, limits)?;

    // Step 2: Clean and prepare staging directory
    let staging_dir = target_dir.with_extension("staging");
    if staging_dir.exists() {
        eprintln!("[Supervisor] Cleaning stale staging directory {:?}", staging_dir);
        wipe_dir(&staging_dir)
            .map_err(|e| format!("Failed to remove stale staging directory {:?}: {}", staging_dir, e))?;
    }
    fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("Failed to create staging directory {:?}: {}", staging_dir, e))?;

    // Step 3: Decompress into staging directory with in-flight validation
    let mut extracted_file_count: usize = 0;
    let mut extracted_bytes: u64 = 0;

    let decompress_res = sevenz_rust::decompress_file_with_extract_fn(
        &archive_path,
        &staging_dir,
        |entry, reader, dest_path| {
            validate_entry_name(entry.name())
                .map_err(|e| sevenz_rust::Error::other(e.to_string()))?;
            validate_entry_attributes(entry)
                .map_err(|e| sevenz_rust::Error::other(e.to_string()))?;

            if !dest_path.starts_with(&staging_dir) {
                return Err(sevenz_rust::Error::other(format!(
                    "Path containment violation: {:?} escapes staging dir {:?}",
                    dest_path, staging_dir
                )));
            }

            if entry.size() > limits.max_single_file_bytes {
                return Err(sevenz_rust::Error::other(format!(
                    "File {:?} size {} exceeds max single file limit {}",
                    entry.name(), entry.size(), limits.max_single_file_bytes
                )));
            }

            extracted_file_count += 1;
            if extracted_file_count > limits.max_entries {
                return Err(sevenz_rust::Error::other(format!(
                    "Archive exceeds max entry count limit {}",
                    limits.max_entries
                )));
            }

            if entry.is_directory() {
                if !dest_path.exists() {
                    fs::create_dir_all(dest_path).map_err(sevenz_rust::Error::io)?;
                }
            } else {
                if let Some(parent) = dest_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(sevenz_rust::Error::io)?;
                    }
                }
                let file = fs::File::create(dest_path)
                    .map_err(|e| sevenz_rust::Error::io_msg(e, dest_path.to_string_lossy().to_string()))?;
                let mut writer = std::io::BufWriter::new(file);
                let mut limited_reader = reader.take(limits.max_single_file_bytes + 1);
                let bytes_copied = std::io::copy(&mut limited_reader, &mut writer)
                    .map_err(sevenz_rust::Error::io)?;
                writer.flush().map_err(sevenz_rust::Error::io)?;

                if bytes_copied > limits.max_single_file_bytes {
                    return Err(sevenz_rust::Error::other(format!(
                        "File {:?} decompressed stream exceeded max single file limit of {} bytes",
                        entry.name(), limits.max_single_file_bytes
                    )));
                }

                extracted_bytes = extracted_bytes.checked_add(bytes_copied)
                    .ok_or_else(|| sevenz_rust::Error::other("Integer overflow in extracted bytes counter"))?;

                if extracted_bytes > limits.max_uncompressed_bytes {
                    return Err(sevenz_rust::Error::other(format!(
                        "Archive exceeded max uncompressed bytes limit of {} bytes during decompression",
                        limits.max_uncompressed_bytes
                    )));
                }
            }
            Ok(true)
        },
    );

    if let Err(e) = decompress_res {
        let _ = wipe_dir(&staging_dir);
        return Err(format!("Failed to decompress 7z archive {:?}: {}", archive_path, e).into());
    }

    // Step 4: Verify expected file exists inside staging directory before swap
    let staging_expected = staging_dir.join(expected_file);
    let nested_expected = staging_dir.join("omnitome-sidecar").join(expected_file);
    let expected_len = staging_expected.metadata().map(|m| m.len())
        .or_else(|_| nested_expected.metadata().map(|m| m.len()))
        .unwrap_or(0);
    if expected_len == 0 {
        let _ = wipe_dir(&staging_dir);
        return Err(format!(
            "Decompressed archive {:?}, but expected verification file {:?} was not found or was 0 bytes in staging",
            archive_path, expected_file
        ).into());
    }

    // Step 5: Verify critical files in staging and collect manifest list
    let critical_files = collect_critical_files_in_staging(&staging_dir, expected_file);
    for check in &critical_files {
        let p1 = staging_dir.join(&check.relative_path);
        let p2 = staging_dir.join("omnitome-sidecar").join(&check.relative_path);
        let found = if p1.exists() {
            Some(p1)
        } else if p2.exists() {
            Some(p2)
        } else {
            None
        };
        match found {
            None => {
                let _ = wipe_dir(&staging_dir);
                return Err(format!("Critical path {:?} missing in staging for archive {:?}", check.relative_path, archive_path).into());
            }
            Some(ref path) => {
                if check.is_directory {
                    if !path.is_dir() {
                        let _ = wipe_dir(&staging_dir);
                        return Err(format!("Critical path {:?} in staging is not a directory", path).into());
                    }
                } else {
                    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                    if size < check.min_size {
                        let _ = wipe_dir(&staging_dir);
                        return Err(format!("Critical file {:?} in staging is undersized ({} < min {})", path, size, check.min_size).into());
                    }
                }
            }
        }
    }

    // Step 6: Write completion manifest into staging directory
    write_manifest(&staging_dir, &archive_path, extracted_file_count, extracted_bytes, critical_files)?;

    // Step 7: Rollback-safe promotion of staging directory into target directory
    promote_staging_dir(&staging_dir, &target_dir)?;

    // Step 8: Post-swap verification
    let target_expected = target_dir.join(expected_file);
    let nested_target_expected = target_dir.join("omnitome-sidecar").join(expected_file);
    if !target_expected.exists() && !nested_target_expected.exists() {
        return Err(format!(
            "Swapped content into {:?}, but verified file {:?} is missing",
            target_dir, expected_file
        ).into());
    }

    // Step 9: Optional archive deletion to reclaim disk space
    if delete_archive_on_success {
        if let Err(e) = fs::remove_file(&archive_path) {
            eprintln!("[Supervisor] Notice: Could not remove archive {:?}: {}", archive_path, e);
        } else {
            eprintln!("[Supervisor] Removed archive {:?} to reclaim disk space", archive_path);
        }
    }

    Ok(())
}

pub fn unpack_sidecar_7z(archive_path: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    unpack_managed_archive_7z(archive_path, target_dir, "omnitome-sidecar.exe", true)
}

fn is_root_or_empty(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed == "C:" || trimmed == r"C:\" || trimmed == "C:/"
        || trimmed.eq_ignore_ascii_case("c:")
        || trimmed.eq_ignore_ascii_case(r"c:\")
        || trimmed.eq_ignore_ascii_case("c:/") {
        return true;
    }
    if trimmed == "/" || trimmed == "\\" {
        return true;
    }
    let bytes = trimmed.as_bytes();
    if bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return true;
    }
    if bytes.len() == 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        return true;
    }
    false
}

fn resolve_node_paths(install_dir: &Path) -> Option<(PathBuf, Vec<String>, PathBuf)> {
    let clean_install_dir = strip_windows_verbatim(install_dir);
    let dir_str = clean_install_dir.to_string_lossy();
    if is_root_or_empty(&dir_str) || !clean_install_dir.exists() {
        return None;
    }

    let node_bin = strip_windows_verbatim(&clean_install_dir.join("resources").join("node").join("node.exe"));
    let server_js = strip_windows_verbatim(&clean_install_dir.join("resources").join("backend").join("server.js"));
    let working_dir = strip_windows_verbatim(&clean_install_dir.join("resources").join("backend"));
    let server_arg = server_js.to_string_lossy().to_string();

    if !is_root_or_empty(&server_arg) && !is_root_or_empty(&node_bin.to_string_lossy()) {
        if node_bin.exists() && server_js.exists() {
            return Some((node_bin, vec![server_arg], working_dir));
        }
    }

    // Fallback: check alongside exe (install_dir without 'resources' nesting)
    let alt_node = strip_windows_verbatim(&clean_install_dir.join("node").join("node.exe"));
    let alt_server = strip_windows_verbatim(&clean_install_dir.join("backend").join("server.js"));
    let alt_working_dir = strip_windows_verbatim(&clean_install_dir.join("backend"));
    let alt_server_arg = alt_server.to_string_lossy().to_string();

    if !is_root_or_empty(&alt_server_arg) && !is_root_or_empty(&alt_node.to_string_lossy()) {
        if alt_node.exists() && alt_server.exists() {
            return Some((alt_node, vec![alt_server_arg], alt_working_dir));
        }
    }

    None
}

fn resolve_bundled_sidecar_exe(install_dir: &Path) -> Option<(PathBuf, Vec<String>, PathBuf)> {
    let clean_dir = strip_windows_verbatim(install_dir);
    let dir_str = clean_dir.to_string_lossy();
    if is_root_or_empty(&dir_str) || !clean_dir.exists() {
        return None;
    }

    let prebundled_res = strip_windows_verbatim(&clean_dir.join("resources").join("sidecar").join("omnitome-sidecar.exe"));
    if !is_root_or_empty(&prebundled_res.to_string_lossy()) && prebundled_res.exists() {
        let working_dir = prebundled_res.parent().unwrap_or(&clean_dir).to_path_buf();
        return Some((prebundled_res, vec![], working_dir));
    }

    let prebundled_direct = strip_windows_verbatim(&clean_dir.join("sidecar").join("omnitome-sidecar.exe"));
    if !is_root_or_empty(&prebundled_direct.to_string_lossy()) && prebundled_direct.exists() {
        let working_dir = prebundled_direct.parent().unwrap_or(&clean_dir).to_path_buf();
        return Some((prebundled_direct, vec![], working_dir));
    }

    None
}

fn get_candidate_sidecar_archives(install_dir: &Path) -> Vec<PathBuf> {
    let clean_dir = strip_windows_verbatim(install_dir);
    let dir_str = clean_dir.to_string_lossy();
    if is_root_or_empty(&dir_str) {
        return Vec::new();
    }
    let mut candidate_archives = Vec::new();
    candidate_archives.push(strip_windows_verbatim(&clean_dir.join("resources").join("sidecar.7z")));
    candidate_archives.push(strip_windows_verbatim(&clean_dir.join("sidecar.7z")));
    candidate_archives
}

fn get_candidate_node_archives(install_dir: &Path) -> Vec<PathBuf> {
    let clean_dir = strip_windows_verbatim(install_dir);
    let dir_str = clean_dir.to_string_lossy();
    if is_root_or_empty(&dir_str) {
        return Vec::new();
    }
    vec![
        strip_windows_verbatim(&clean_dir.join("resources").join("node.7z")),
        strip_windows_verbatim(&clean_dir.join("node.7z")),
    ]
}

fn get_candidate_backend_archives(install_dir: &Path) -> Vec<PathBuf> {
    let clean_dir = strip_windows_verbatim(install_dir);
    let dir_str = clean_dir.to_string_lossy();
    if is_root_or_empty(&dir_str) {
        return Vec::new();
    }
    vec![
        strip_windows_verbatim(&clean_dir.join("resources").join("backend.7z")),
        strip_windows_verbatim(&clean_dir.join("backend.7z")),
    ]
}

fn find_sidecar_executable(_app: &tauri::AppHandle, _project_root: &Path) -> Result<(PathBuf, Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    // 1. In dev mode (debug_assertions), prefer venv python main.py or local frozen sidecar
    #[cfg(debug_assertions)]
    {
        // 1. Check if we are in project root
        let candidate_venv = strip_windows_verbatim(&_project_root.join("sidecars").join("python").join(".venv").join("Scripts").join("python.exe"));
        let candidate_script = strip_windows_verbatim(&_project_root.join("sidecars").join("python").join("main.py"));

        // 2. Check the parent directory (project root if running from src-tauri)
        let parent_venv = strip_windows_verbatim(&_project_root.join("..").join("sidecars").join("python").join(".venv").join("Scripts").join("python.exe"));
        let parent_script = strip_windows_verbatim(&_project_root.join("..").join("sidecars").join("python").join("main.py"));

        if candidate_venv.exists() && candidate_script.exists() {
            return Ok((candidate_venv, vec![candidate_script.to_string_lossy().to_string()], strip_windows_verbatim(_project_root)));
        }
        if parent_venv.exists() && parent_script.exists() {
            let parent_root = _project_root.join("..").canonicalize().unwrap_or_else(|_| _project_root.join(".."));
            return Ok((parent_venv, vec![parent_script.to_string_lossy().to_string()], strip_windows_verbatim(&parent_root)));
        }

        let local_frozen = strip_windows_verbatim(&_project_root.join("dist-sidecar").join("omnitome-sidecar").join("omnitome-sidecar.exe"));
        if local_frozen.exists() {
            let working_dir = local_frozen.parent().unwrap_or(_project_root).to_path_buf();
            return Ok((local_frozen, vec![], working_dir));
        }

        let parent_frozen = strip_windows_verbatim(&_project_root.join("..").join("dist-sidecar").join("omnitome-sidecar").join("omnitome-sidecar.exe"));
        if parent_frozen.exists() {
            let parent_root = _project_root.join("..").canonicalize().unwrap_or_else(|_| _project_root.join(".."));
            let working_dir = parent_frozen.parent().unwrap_or(&parent_root).to_path_buf();
            return Ok((parent_frozen, vec![], working_dir));
        }
    }

    // 2. In release/packaged mode:
    let app_data = strip_windows_verbatim(&get_app_data_dir());
    let local_sidecar_dir = strip_windows_verbatim(&app_data.join("sidecar"));
    let local_sidecar_exe = strip_windows_verbatim(&local_sidecar_dir.join("omnitome-sidecar.exe"));
    let nested_local_exe = strip_windows_verbatim(&local_sidecar_dir.join("omnitome-sidecar").join("omnitome-sidecar.exe"));

    let exe_path = strip_windows_verbatim(&std::env::current_exe()?);
    let install_dir = exe_path.parent().ok_or("Failed to get install dir")?;
    let install_dir = strip_windows_verbatim(install_dir);
    let install_dir_str = install_dir.to_string_lossy();
    if is_root_or_empty(&install_dir_str) {
        return Err(format!("Installation directory resolves to root or empty: {:?}", install_dir).into());
    }
    eprintln!("[Supervisor] Found installation directory for sidecar: {:?}", install_dir);

    // If missing or updated, check candidate locations for bundled sidecar.7z
    let mut candidate_archives = get_candidate_sidecar_archives(&install_dir);
    #[cfg(debug_assertions)]
    {
        candidate_archives.push(strip_windows_verbatim(&_project_root.join("resources").join("sidecar.7z")));
        candidate_archives.push(strip_windows_verbatim(&_project_root.join("..").join("resources").join("sidecar.7z")));
    }

    for archive_path in &candidate_archives {
        if archive_path.exists() {
            unpack_sidecar_7z(archive_path, &local_sidecar_dir)?;
            if local_sidecar_exe.exists() {
                return Ok((local_sidecar_exe, vec![], local_sidecar_dir));
            }
            if nested_local_exe.exists() {
                let nested_dir = strip_windows_verbatim(&local_sidecar_dir.join("omnitome-sidecar"));
                return Ok((nested_local_exe, vec![], nested_dir));
            }
            return Err(format!(
                "Unpacked sidecar archive {:?} into {:?}, but executable {:?} was not found",
                archive_path, local_sidecar_dir, local_sidecar_exe
            ).into());
        }
    }

    if local_sidecar_exe.exists() {
        return Ok((local_sidecar_exe, vec![], local_sidecar_dir));
    }
    if nested_local_exe.exists() {
        let nested_dir = strip_windows_verbatim(&local_sidecar_dir.join("omnitome-sidecar"));
        return Ok((nested_local_exe, vec![], nested_dir));
    }

    // 3. Pre-bundled sidecar executable (if pre-extracted in install_dir/resources or alongside exe)
    if let Some(paths) = resolve_bundled_sidecar_exe(&install_dir) {
        return Ok(paths);
    }

    let checked_archive = install_dir.join("resources").join("sidecar.7z");
    let checked_prebundled = install_dir.join("resources").join("sidecar").join("omnitome-sidecar.exe");

    Err(format!(
        "Failed to locate Omnitome sidecar executable. Checked local directory ({:?}), archive ({:?}), and pre-bundled executable ({:?})",
        local_sidecar_exe, checked_archive, checked_prebundled
    ).into())
}

fn find_node_executable(_app: &tauri::AppHandle, _project_root: &Path) -> Result<(PathBuf, Vec<String>, PathBuf), Box<dyn std::error::Error>> {
    // 1. In dev mode (debug_assertions), prefer project backend and local/nvm node
    #[cfg(debug_assertions)]
    {
        let candidate_server = strip_windows_verbatim(&_project_root.join("backend").join("server.js"));
        let parent_server = strip_windows_verbatim(&_project_root.join("..").join("backend").join("server.js"));

        let (server_js, working_dir) = if candidate_server.exists() {
            (candidate_server, strip_windows_verbatim(_project_root))
        } else if parent_server.exists() {
            let parent_root = _project_root.join("..").canonicalize().unwrap_or_else(|_| _project_root.join(".."));
            (parent_server, strip_windows_verbatim(&parent_root))
        } else {
            (candidate_server, strip_windows_verbatim(_project_root))
        };

        let nvm_node = PathBuf::from(r"C:\Users\Godsa\AppData\Local\Author Software\nvm\installs\v22.23.2\node.exe");
        if nvm_node.exists() && server_js.exists() {
            return Ok((strip_windows_verbatim(&nvm_node), vec![server_js.to_string_lossy().to_string()], working_dir));
        }
        if server_js.exists() {
            return Ok(("node".into(), vec![server_js.to_string_lossy().to_string()], working_dir));
        }
    }

    // 2. In release/packaged mode:
    let app_data = strip_windows_verbatim(&get_app_data_dir());
    let local_node_dir = strip_windows_verbatim(&app_data.join("node"));
    let local_node_bin = strip_windows_verbatim(&local_node_dir.join("node.exe"));
    let local_backend_dir = strip_windows_verbatim(&app_data.join("backend"));
    let local_server_js = strip_windows_verbatim(&local_backend_dir.join("server.js"));

    let exe_path = strip_windows_verbatim(&std::env::current_exe()?);
    let install_dir = exe_path.parent().ok_or("Failed to get install dir")?;
    let install_dir = strip_windows_verbatim(install_dir);
    let install_dir_str = install_dir.to_string_lossy();
    if is_root_or_empty(&install_dir_str) {
        return Err(format!("Installation directory resolves to root or empty: {:?}", install_dir).into());
    }
    eprintln!("[Supervisor] Found installation directory: {:?}", install_dir);

    // Check for candidate archives
    let mut candidate_node_archives = get_candidate_node_archives(&install_dir);
    let mut candidate_backend_archives = get_candidate_backend_archives(&install_dir);
    #[cfg(debug_assertions)]
    {
        candidate_node_archives.push(strip_windows_verbatim(&_project_root.join("resources").join("node.7z")));
        candidate_node_archives.push(strip_windows_verbatim(&_project_root.join("..").join("resources").join("node.7z")));
        candidate_backend_archives.push(strip_windows_verbatim(&_project_root.join("resources").join("backend.7z")));
        candidate_backend_archives.push(strip_windows_verbatim(&_project_root.join("..").join("resources").join("backend.7z")));
    }

    for archive in &candidate_node_archives {
        if archive.exists() {
            unpack_managed_archive_7z(archive, &local_node_dir, "node.exe", false)?;
            break;
        }
    }

    for archive in &candidate_backend_archives {
        if archive.exists() {
            unpack_managed_archive_7z(archive, &local_backend_dir, "server.js", false)?;
            break;
        }
    }

    if local_node_bin.exists() && local_server_js.exists() {
        return Ok((local_node_bin, vec![local_server_js.to_string_lossy().to_string()], local_backend_dir));
    }

    if let Some(paths) = resolve_node_paths(&install_dir) {
        return Ok(paths);
    }

    let node_bin = install_dir.join("resources").join("node").join("node.exe");
    let server_js = install_dir.join("resources").join("backend").join("server.js");
    let alt_node = install_dir.join("node").join("node.exe");
    let alt_server = install_dir.join("backend").join("server.js");

    Err(format!(
        "Could not find or unpack Node.js and backend/server.js. Checked: local_node={:?}, local_server={:?}, node_bin={:?}, server_js={:?}, alt_node={:?}, alt_server={:?}",
        local_node_bin, local_server_js, node_bin, server_js, alt_node, alt_server
    ).into())
}

fn stream_output(
    prefix: &'static str,
    stream: impl std::io::Read + Send + 'static,
    log_file: PathBuf,
    expected_service: &'static str,
    port_sender: Option<Sender<u16>>,
) {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        let regex = Regex::new(r"^__OMNITOME_BOOT__\s+([a-zA-Z0-9_\-]+)\s+port=(\d+)\s*$").unwrap();
        let mut sender = port_sender;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .ok();

        for line in reader.lines() {
            if let Ok(line) = line {
                let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                let log_entry = format!("[{}] [{}] {}\n", ts, prefix, line);

                if let Some(ref mut f) = file {
                    let _ = f.write_all(log_entry.as_bytes());
                    let _ = f.flush();
                }

                #[cfg(debug_assertions)]
                eprintln!("[{}] {}", prefix, line);

                // Check for sentinel line
                if let Some(s) = sender.take() {
                    let trimmed = line.trim();
                    if let Some(caps) = regex.captures(trimmed) {
                        let service = caps.get(1).map_or("", |m| m.as_str());
                        let port_str = caps.get(2).map_or("", |m| m.as_str());
                        if service == expected_service {
                            if let Ok(port) = port_str.parse::<u16>() {
                                let _ = s.send(port);
                            } else {
                                sender = Some(s);
                            }
                        } else {
                            sender = Some(s);
                        }
                    } else {
                        sender = Some(s);
                    }
                }
            }
        }
    });
}

fn setup_supervisor(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = strip_windows_verbatim(&get_app_data_dir());
    let logs_dir = strip_windows_verbatim(&app_dir.join("logs"));
    let models_dir = strip_windows_verbatim(&app_dir.join("models"));

    // 1. Clean stale processes from previous crashes
    cleanup_stale_pids(&app_dir);

    // 2. Setup log paths and rotation
    let sidecar_log = strip_windows_verbatim(&logs_dir.join("sidecar.log"));
    rotate_log_if_needed(&sidecar_log);
    let node_log = strip_windows_verbatim(&logs_dir.join("node.log"));
    rotate_log_if_needed(&node_log);

    #[cfg(debug_assertions)]
    let project_root = strip_windows_verbatim(&find_project_root());
    #[cfg(not(debug_assertions))]
    let project_root = PathBuf::new();

    // 3. Spawn Python sidecar
    let (sidecar_bin, mut sidecar_args, sidecar_cwd) = find_sidecar_executable(app, &project_root)?;
    let sidecar_bin = strip_windows_verbatim(&sidecar_bin);
    let sidecar_cwd = strip_windows_verbatim(&sidecar_cwd);
    sidecar_args.extend(["--port".into(), "0".into()]);

    let mut sidecar_cmd = Command::new(&sidecar_bin);
    sidecar_cmd.args(&sidecar_args);
    sidecar_cmd.current_dir(&sidecar_cwd);
    sidecar_cmd.env("OMNITOME_MODEL_DIR", &models_dir);
    sidecar_cmd.env("PYTHONUNBUFFERED", "1");
    sidecar_cmd.stdout(Stdio::piped());
    sidecar_cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    sidecar_cmd.creation_flags(CREATE_NO_WINDOW);

    let mut sidecar_child = sidecar_cmd.spawn()
        .map_err(|e| format!("Failed to spawn Python sidecar ({:?}): {}", sidecar_bin, e))?;

    let sidecar_pid = sidecar_child.id();

    let (sidecar_port_tx, sidecar_port_rx) = channel();
    let sidecar_stdout = sidecar_child.stdout.take().ok_or("Failed to capture sidecar stdout")?;
    let sidecar_stderr = sidecar_child.stderr.take().ok_or("Failed to capture sidecar stderr")?;

    stream_output("sidecar", sidecar_stdout, sidecar_log.clone(), "python-sidecar", Some(sidecar_port_tx));
    stream_output("sidecar:err", sidecar_stderr, sidecar_log.clone(), "python-sidecar", None);

    // Await sidecar boot sentinel with responsive child exit detection
    let start_sidecar = std::time::Instant::now();
    let timeout = Duration::from_secs(30);
    let sidecar_port = loop {
        match sidecar_port_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(port) => break port,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(Some(status)) = sidecar_child.try_wait() {
                    return Err(format!(
                        "Python sidecar exited prematurely with {}. Check {:?} for details.",
                        status, sidecar_log
                    ).into());
                }
                if start_sidecar.elapsed() >= timeout {
                    let _ = sidecar_child.kill();
                    return Err(format!(
                        "Timeout (30s) waiting for Python sidecar boot handshake sentinel. Check {:?} for details.",
                        sidecar_log
                    ).into());
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let status_msg = match sidecar_child.try_wait() {
                    Ok(Some(status)) => format!("exited with {}", status),
                    _ => "stdout/stderr stream closed unexpectedly".into(),
                };
                return Err(format!(
                    "Python sidecar stream disconnected ({}). Check {:?} for details.",
                    status_msg, sidecar_log
                ).into());
            }
        }
    };

    // 4. Spawn Node backend with OMNITOME_SIDECAR_PORT
    let (node_bin, mut node_args, node_cwd) = find_node_executable(app, &project_root)?;
    let node_bin = strip_windows_verbatim(&node_bin);
    let node_cwd = strip_windows_verbatim(&node_cwd);

    if node_args.is_empty() {
        return Err("Node script arguments are empty before extending flags".into());
    }
    // Clean Windows verbatim prefix from script argument
    node_args[0] = strip_windows_verbatim(Path::new(&node_args[0])).to_string_lossy().to_string();

    if is_root_or_empty(&node_args[0]) {
        return Err(format!("Node script argument resolves to root or empty before extending flags: {:?}", node_args[0]).into());
    }

    node_args.extend(["--port".into(), "0".into()]);

    if node_args.is_empty() {
        return Err("Node script arguments are empty after extending flags".into());
    }
    if is_root_or_empty(&node_args[0]) {
        return Err(format!("Node script argument resolves to root or empty after extending flags: {:?}", node_args[0]).into());
    }

    let mut node_cmd = Command::new(&node_bin);
    node_cmd.args(&node_args);
    node_cmd.current_dir(&node_cwd);
    node_cmd.env("OMNITOME_SIDECAR_PORT", sidecar_port.to_string());
    node_cmd.env("PYTHON_SIDECAR_URL", format!("http://127.0.0.1:{}", sidecar_port));
    node_cmd.env("OMNITOME_MODEL_DIR", &models_dir);

    #[cfg(not(debug_assertions))]
    {
        let storage_dir = strip_windows_verbatim(&app_dir);
        let db_path = strip_windows_verbatim(&app_dir.join("data.db"));
        let settings_file_path = strip_windows_verbatim(&app_dir.join("settings.json"));
        node_cmd.env("STORAGE_DIR", &storage_dir);
        node_cmd.env("DB_PATH", &db_path);
        node_cmd.env("SETTINGS_FILE_PATH", &settings_file_path);
    }

    node_cmd.stdout(Stdio::piped());
    node_cmd.stderr(Stdio::piped());

    #[cfg(windows)]
    node_cmd.creation_flags(CREATE_NO_WINDOW);

    eprintln!("[Supervisor] About to spawn Node: {:?} {:?} in {:?}", node_bin, node_args, node_cwd);

    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&node_log) {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let _ = writeln!(f, "[{}] [supervisor] Spawning node: {:?} {:?} in {:?}", ts, node_bin, node_args, node_cwd);
        let _ = f.flush();
    }

    let mut node_child = node_cmd.spawn()
        .map_err(|e| format!("Failed to spawn Node backend ({:?}): {}", node_bin, e))?;

    let node_pid = node_child.id();

    let (node_port_tx, node_port_rx) = channel();
    let node_stdout = node_child.stdout.take().ok_or("Failed to capture node stdout")?;
    let node_stderr = node_child.stderr.take().ok_or("Failed to capture node stderr")?;

    stream_output("node", node_stdout, node_log.clone(), "node-backend", Some(node_port_tx));
    stream_output("node:err", node_stderr, node_log.clone(), "node-backend", None);

    // Await Node backend boot sentinel with responsive child exit detection
    let start_node = std::time::Instant::now();
    let node_port = loop {
        match node_port_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(port) => break port,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if let Ok(Some(status)) = node_child.try_wait() {
                    let _ = sidecar_child.kill();
                    return Err(format!(
                        "Node backend exited prematurely with {}. Check {:?} for details.",
                        status, node_log
                    ).into());
                }
                if start_node.elapsed() >= timeout {
                    let _ = node_child.kill();
                    let _ = sidecar_child.kill();
                    return Err(format!(
                        "Timeout (30s) waiting for Node backend boot handshake sentinel. Check {:?} for details.",
                        node_log
                    ).into());
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let _ = node_child.kill();
                let _ = sidecar_child.kill();
                let status_msg = match node_child.try_wait() {
                    Ok(Some(status)) => format!("exited with {}", status),
                    _ => "stdout/stderr stream closed unexpectedly".into(),
                };
                return Err(format!(
                    "Node backend stream disconnected ({}). Check {:?} for details.",
                    status_msg, node_log
                ).into());
            }
        }
    };

    // 5. Write atomic PID record
    write_atomic_pids(&app_dir, sidecar_pid, node_pid);

    // Register process manager in app state
    app.manage(ProcessManager {
        sidecar: Mutex::new(Some(sidecar_child)),
        node: Mutex::new(Some(node_child)),
        sidecar_pid,
        node_pid,
        app_dir,
    });

    // 6. Set dynamic ports on pre-configured main window
    app.manage(DynamicPorts {
        node_port,
        sidecar_port,
    });

    let init_script = format!(
        "try {{ sessionStorage.setItem('omnitome_ports', JSON.stringify({{ nodePort: {}, sidecarPort: {} }})); }} catch(e) {{}} window.__OMNITOME__ = Object.freeze({{ nodePort: {}, sidecarPort: {} }}); window.dispatchEvent(new CustomEvent('omnitome:ready', {{ detail: window.__OMNITOME__ }}));",
        node_port, sidecar_port, node_port, sidecar_port
    );

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.eval(&init_script);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .on_page_load(|webview, payload| {
            if webview.label() == "main" && payload.event() == tauri::webview::PageLoadEvent::Finished {
                if let Some(ports) = webview.try_state::<DynamicPorts>() {
                    let script = format!(
                        "try {{ sessionStorage.setItem('omnitome_ports', JSON.stringify({{ nodePort: {}, sidecarPort: {} }})); }} catch(e) {{}} window.__OMNITOME__ = Object.freeze({{ nodePort: {}, sidecarPort: {} }}); window.dispatchEvent(new CustomEvent('omnitome:ready', {{ detail: window.__OMNITOME__ }}));",
                        ports.node_port, ports.sidecar_port, ports.node_port, ports.sidecar_port
                    );
                    let _ = webview.eval(&script);
                }
            }
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { .. } = event {
                    if let Some(pm) = window.app_handle().try_state::<ProcessManager>() {
                        pm.shutdown();
                    }
                }
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                match setup_supervisor(&app_handle) {
                    Ok(()) => {
                        if let Some(splash) = app_handle.get_webview_window("splashscreen") {
                            let _ = splash.close();
                        }
                        if let Some(main) = app_handle.get_webview_window("main") {
                            let _ = main.show();
                            let _ = main.set_focus();
                        }
                    }
                    Err(e) => {
                        if let Some(splash) = app_handle.get_webview_window("splashscreen") {
                            let _ = splash.close();
                        }
                        let err_str = e.to_string();
                        log_crash(&err_str);
                        show_error_dialog("Omnitome Startup Error", &err_str);
                    }
                }
            });
            Ok(())
        });

    let app = match builder.build(tauri::generate_context!()) {
        Ok(app) => app,
        Err(err) => {
            let err_str = format!("Fatal error building Tauri application: {}", err);
            log_crash(&err_str);
            show_error_dialog("Omnitome Fatal Error", &err_str);
            std::process::exit(1);
        }
    };

    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                if let Some(pm) = app_handle.try_state::<ProcessManager>() {
                    pm.shutdown();
                }
            }
            tauri::RunEvent::WindowEvent { label, event: tauri::WindowEvent::CloseRequested { .. }, .. } => {
                if label == "main" {
                    if let Some(pm) = app_handle.try_state::<ProcessManager>() {
                        pm.shutdown();
                    }
                }
            }
            tauri::RunEvent::WindowEvent { label, event: tauri::WindowEvent::Destroyed, .. } => {
                if label == "main" {
                    if let Some(pm) = app_handle.try_state::<ProcessManager>() {
                        pm.shutdown();
                    }
                }
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_unpack_sidecar_7z_success() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_success");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src_sidecar");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("omnitome-sidecar.exe"), b"mock binary content").unwrap();
        fs::write(src_dir.join("config.json"), b"mock config").unwrap();

        let archive_path = temp_dir.join("sidecar.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();
        assert!(archive_path.exists());

        let target_dir = temp_dir.join("sidecar");
        let staging_dir = target_dir.with_extension("staging");

        let res = unpack_sidecar_7z(&archive_path, &target_dir);
        assert!(res.is_ok(), "unpack_sidecar_7z failed: {:?}", res.err());

        assert!(target_dir.join("omnitome-sidecar.exe").exists());
        assert!(target_dir.join("config.json").exists());
        assert!(!staging_dir.exists(), "staging directory should not exist after rename");
        assert!(!archive_path.exists(), "archive file should have been deleted");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unpack_sidecar_7z_cleans_stale_staging() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_stale");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src_sidecar");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("omnitome-sidecar.exe"), b"mock binary content").unwrap();

        let archive_path = temp_dir.join("sidecar.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();

        let target_dir = temp_dir.join("sidecar");
        let staging_dir = target_dir.with_extension("staging");
        fs::create_dir_all(&staging_dir).unwrap();
        fs::write(staging_dir.join("stale_garbage.txt"), b"stale data").unwrap();

        let res = unpack_sidecar_7z(&archive_path, &target_dir);
        assert!(res.is_ok(), "unpack_sidecar_7z failed: {:?}", res.err());

        assert!(target_dir.join("omnitome-sidecar.exe").exists());
        assert!(!target_dir.join("stale_garbage.txt").exists(), "stale staging contents should be wiped");
        assert!(!staging_dir.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unpack_sidecar_7z_corrupt_archive_fails_cleanly() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_corrupt");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let archive_path = temp_dir.join("corrupt.7z");
        fs::write(&archive_path, b"not a real 7z archive file").unwrap();

        let target_dir = temp_dir.join("sidecar");
        let staging_dir = target_dir.with_extension("staging");

        let res = unpack_sidecar_7z(&archive_path, &target_dir);
        assert!(res.is_err(), "unpack_sidecar_7z should fail on corrupt archive");
        assert!(!staging_dir.exists(), "staging dir should be cleaned up on error");
        assert!(!target_dir.exists(), "target dir should not have been created");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unpack_sidecar_7z_cleans_stale_staging_with_readonly_file() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_readonly");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src_sidecar");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("omnitome-sidecar.exe"), b"mock binary content").unwrap();

        let archive_path = temp_dir.join("sidecar.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();

        let target_dir = temp_dir.join("sidecar");
        let staging_dir = target_dir.with_extension("staging");
        fs::create_dir_all(&staging_dir).unwrap();
        let ro_file = staging_dir.join("readonly_stale.txt");
        fs::write(&ro_file, b"cannot delete me easily").unwrap();
        let mut perms = fs::metadata(&ro_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&ro_file, perms).unwrap();

        let res = unpack_sidecar_7z(&archive_path, &target_dir);
        assert!(res.is_ok(), "unpack_sidecar_7z failed with readonly file in staging: {:?}", res.err());

        assert!(target_dir.join("omnitome-sidecar.exe").exists());
        assert!(!staging_dir.exists());

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_unpack_sidecar_7z_created_by_7z_exe() {
        let candidate_7z = [
            r"C:\Program Files\7-Zip\7z.exe",
            r"C:\Program Files (x86)\7-Zip\7z.exe",
        ];
        let mut seven_zip = None;
        for c in &candidate_7z {
            if Path::new(c).exists() {
                seven_zip = Some(*c);
                break;
            }
        }
        let Some(seven_zip_path) = seven_zip else {
            eprintln!("Skipping test_unpack_sidecar_7z_created_by_7z_exe: 7z.exe not found");
            return;
        };

        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_7z_exe");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src_sidecar");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("omnitome-sidecar.exe"), b"real 7z binary content").unwrap();
        fs::create_dir_all(src_dir.join("_internal")).unwrap();
        fs::write(src_dir.join("_internal").join("test_dll.dll"), b"mock internal dll").unwrap();

        let archive_path = temp_dir.join("sidecar.7z");
        let mut cmd = Command::new(seven_zip_path);
        cmd.args(["a", "-t7z", "-mx=5", &archive_path.to_string_lossy(), "*"]);
        cmd.current_dir(&src_dir);
        let status = cmd.status().expect("failed to execute 7z.exe");
        assert!(status.success(), "7z.exe compression failed");
        assert!(archive_path.exists());

        let target_dir = temp_dir.join("sidecar");
        let res = unpack_sidecar_7z(&archive_path, &target_dir);
        assert!(res.is_ok(), "unpack_sidecar_7z failed on 7z.exe archive: {:?}", res.err());

        assert!(target_dir.join("omnitome-sidecar.exe").exists());
        assert!(target_dir.join("_internal").join("test_dll.dll").exists());
        assert!(!archive_path.exists(), "archive file should have been removed to save space");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_log_crash() {
        let unique_marker = format!("TEST_CRASH_MARKER_{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(0));
        log_crash(&unique_marker);

        let crash_log = get_app_data_dir().join("logs").join("crash.log");
        assert!(crash_log.exists());
        let content = fs::read_to_string(&crash_log).unwrap();
        assert!(content.contains(&unique_marker));
    }

    #[test]
    fn test_resolve_node_paths_install_dir_valid() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_node_install_dir");
        let _ = wipe_dir(&temp_dir);
        let node_dir = temp_dir.join("resources").join("node");
        let backend_dir = temp_dir.join("resources").join("backend");
        fs::create_dir_all(&node_dir).unwrap();
        fs::create_dir_all(&backend_dir).unwrap();
        fs::write(node_dir.join("node.exe"), b"mock node").unwrap();
        fs::write(backend_dir.join("server.js"), b"mock server").unwrap();

        let resolved = resolve_node_paths(&temp_dir);
        assert!(resolved.is_some());
        let (bin, args, cwd) = resolved.unwrap();
        assert_eq!(bin, node_dir.join("node.exe"));
        assert_eq!(args, vec![backend_dir.join("server.js").to_string_lossy().to_string()]);
        assert_eq!(cwd, backend_dir);

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_resolve_node_paths_fallback_alongside_exe() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_node_fallback");
        let _ = wipe_dir(&temp_dir);
        let node_dir = temp_dir.join("node");
        let backend_dir = temp_dir.join("backend");
        fs::create_dir_all(&node_dir).unwrap();
        fs::create_dir_all(&backend_dir).unwrap();
        fs::write(node_dir.join("node.exe"), b"mock node").unwrap();
        fs::write(backend_dir.join("server.js"), b"mock server").unwrap();

        let resolved = resolve_node_paths(&temp_dir);
        assert!(resolved.is_some());
        let (bin, args, cwd) = resolved.unwrap();
        assert_eq!(bin, node_dir.join("node.exe"));
        assert_eq!(args, vec![backend_dir.join("server.js").to_string_lossy().to_string()]);
        assert_eq!(cwd, backend_dir);

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_strip_windows_verbatim() {
        assert_eq!(
            strip_windows_verbatim(Path::new(r"\\?\C:\Program Files\Omnitome\server.js")),
            PathBuf::from(r"C:\Program Files\Omnitome\server.js")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new(r"\\?\UNC\server\share\file.txt")),
            PathBuf::from(r"\\server\share\file.txt")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new(r"\\?\unc\server\share\file.txt")),
            PathBuf::from(r"\\server\share\file.txt")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new(r"//?/UNC/server/share/file.txt")),
            PathBuf::from(r"//server/share/file.txt")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new(r"\\?/C:/Program Files/Omnitome/server.js")),
            PathBuf::from(r"C:/Program Files/Omnitome/server.js")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new(r"//?/C:/Program Files/Omnitome/server.js")),
            PathBuf::from(r"C:/Program Files/Omnitome/server.js")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new(r"C:\Users\Omnitome")),
            PathBuf::from(r"C:\Users\Omnitome")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new("relative/path/test.js")),
            PathBuf::from("relative/path/test.js")
        );
        assert_eq!(
            strip_windows_verbatim(Path::new("")),
            PathBuf::from("")
        );
    }

    #[test]
    fn test_resolve_node_paths_strips_windows_verbatim() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_node_verbatim");
        let _ = wipe_dir(&temp_dir);
        let node_dir = temp_dir.join("resources").join("node");
        let backend_dir = temp_dir.join("resources").join("backend");
        fs::create_dir_all(&node_dir).unwrap();
        fs::create_dir_all(&backend_dir).unwrap();
        fs::write(node_dir.join("node.exe"), b"mock node").unwrap();
        fs::write(backend_dir.join("server.js"), b"mock server").unwrap();

        let verbatim_path = PathBuf::from(format!(r"\\?\{}", temp_dir.to_string_lossy()));
        let resolved = resolve_node_paths(&verbatim_path);
        assert!(resolved.is_some());
        let (bin, args, cwd) = resolved.unwrap();
        assert!(!bin.to_string_lossy().starts_with(r"\\?\"), "node binary should not have verbatim prefix: {:?}", bin);
        assert!(!args[0].starts_with(r"\\?\"), "server arg should not have verbatim prefix: {:?}", args[0]);
        assert!(!cwd.to_string_lossy().starts_with(r"\\?\"), "working dir should not have verbatim prefix: {:?}", cwd);

        assert_eq!(bin, node_dir.join("node.exe"));
        assert_eq!(args, vec![backend_dir.join("server.js").to_string_lossy().to_string()]);
        assert_eq!(cwd, backend_dir);

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_resolve_node_paths_partial_or_missing_fails() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_node_missing");
        let _ = wipe_dir(&temp_dir);
        let node_dir = temp_dir.join("resources").join("node");
        fs::create_dir_all(&node_dir).unwrap();
        fs::write(node_dir.join("node.exe"), b"mock node").unwrap();

        // Node exists but server.js is missing
        assert!(resolve_node_paths(&temp_dir).is_none());

        // server.js exists but node.exe is missing
        let temp_dir2 = std::env::temp_dir().join("omnitome_test_node_missing_exe");
        let _ = wipe_dir(&temp_dir2);
        let backend_dir2 = temp_dir2.join("resources").join("backend");
        fs::create_dir_all(&backend_dir2).unwrap();
        fs::write(backend_dir2.join("server.js"), b"mock server").unwrap();
        assert!(resolve_node_paths(&temp_dir2).is_none());
        let _ = wipe_dir(&temp_dir2);

        // Entire directory does not exist
        let non_existent = temp_dir.join("does_not_exist");
        assert!(resolve_node_paths(&non_existent).is_none());

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_is_root_or_empty() {
        assert!(is_root_or_empty(""));
        assert!(is_root_or_empty("   "));
        assert!(is_root_or_empty("C:"));
        assert!(is_root_or_empty(r"C:\"));
        assert!(is_root_or_empty("C:/"));
        assert!(is_root_or_empty("c:"));
        assert!(is_root_or_empty(r"c:\"));
        assert!(is_root_or_empty("c:/"));
        assert!(is_root_or_empty("D:"));
        assert!(is_root_or_empty(r"D:\"));
        assert!(is_root_or_empty("D:/"));
        assert!(is_root_or_empty("/"));
        assert!(is_root_or_empty("\\"));
        assert!(!is_root_or_empty(r"C:\backend\server.js"));
        assert!(!is_root_or_empty("backend/server.js"));
        assert!(!is_root_or_empty(r"C:\Program Files\Omnitome"));
    }

    #[test]
    fn test_resolve_node_paths_rejects_root_and_empty() {
        assert!(resolve_node_paths(Path::new("")).is_none());
        assert!(resolve_node_paths(Path::new("   ")).is_none());
        assert!(resolve_node_paths(Path::new("C:")).is_none());
        assert!(resolve_node_paths(Path::new(r"C:\")).is_none());
        assert!(resolve_node_paths(Path::new("C:/")).is_none());
        assert!(resolve_node_paths(Path::new("c:")).is_none());
        assert!(resolve_node_paths(Path::new(r"c:\")).is_none());
        assert!(resolve_node_paths(Path::new("c:/")).is_none());
        assert!(resolve_node_paths(Path::new("D:")).is_none());
        assert!(resolve_node_paths(Path::new(r"D:\")).is_none());
        assert!(resolve_node_paths(Path::new("D:/")).is_none());
        assert!(resolve_node_paths(Path::new("/")).is_none());
        assert!(resolve_node_paths(Path::new("\\")).is_none());
    }

    #[test]
    fn test_resolve_bundled_sidecar_exe_locations() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_sidecar_bundled");
        let _ = wipe_dir(&temp_dir);

        // 1. install_dir/resources/sidecar/omnitome-sidecar.exe hit
        let sidecar_res_dir = temp_dir.join("resources").join("sidecar");
        fs::create_dir_all(&sidecar_res_dir).unwrap();
        fs::write(sidecar_res_dir.join("omnitome-sidecar.exe"), b"mock exe").unwrap();
        let res = resolve_bundled_sidecar_exe(&temp_dir);
        assert!(res.is_some());
        let (bin, _, cwd) = res.unwrap();
        assert_eq!(bin, sidecar_res_dir.join("omnitome-sidecar.exe"));
        assert_eq!(cwd, sidecar_res_dir);

        // 2. install_dir/sidecar/omnitome-sidecar.exe fallback hit
        let temp_dir2 = std::env::temp_dir().join("omnitome_test_sidecar_bundled_fallback");
        let _ = wipe_dir(&temp_dir2);
        let sidecar_direct_dir = temp_dir2.join("sidecar");
        fs::create_dir_all(&sidecar_direct_dir).unwrap();
        fs::write(sidecar_direct_dir.join("omnitome-sidecar.exe"), b"mock exe").unwrap();
        let res2 = resolve_bundled_sidecar_exe(&temp_dir2);
        assert!(res2.is_some());
        let (bin2, _, cwd2) = res2.unwrap();
        assert_eq!(bin2, sidecar_direct_dir.join("omnitome-sidecar.exe"));
        assert_eq!(cwd2, sidecar_direct_dir);

        // 3. Verbatim prefix stripping
        let verbatim_temp = PathBuf::from(format!(r"\\?\{}", temp_dir.to_string_lossy()));
        let res3 = resolve_bundled_sidecar_exe(&verbatim_temp);
        assert!(res3.is_some());
        let (bin3, _, cwd3) = res3.unwrap();
        assert!(!bin3.to_string_lossy().starts_with(r"\\?\"));
        assert!(!cwd3.to_string_lossy().starts_with(r"\\?\"));

        // 4. Root and empty checks
        assert!(resolve_bundled_sidecar_exe(Path::new("")).is_none());
        assert!(resolve_bundled_sidecar_exe(Path::new(r"C:\")).is_none());

        let _ = wipe_dir(&temp_dir);
        let _ = wipe_dir(&temp_dir2);
    }

    #[test]
    fn test_get_candidate_node_archives_strict_order() {
        let install_dir = PathBuf::from(r"C:\Program Files\Omnitome");
        let archives = get_candidate_node_archives(&install_dir);
        assert_eq!(archives.len(), 2);
        assert_eq!(archives[0], install_dir.join("resources").join("node.7z"));
        assert_eq!(archives[1], install_dir.join("node.7z"));
    }

    #[test]
    fn test_get_candidate_backend_archives_strict_order() {
        let install_dir = PathBuf::from(r"C:\Program Files\Omnitome");
        let archives = get_candidate_backend_archives(&install_dir);
        assert_eq!(archives.len(), 2);
        assert_eq!(archives[0], install_dir.join("resources").join("backend.7z"));
        assert_eq!(archives[1], install_dir.join("backend.7z"));
    }

    #[test]
    fn test_extraction_lock_acquire_and_drop() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_lock_acquire");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        {
            let lock = ExtractionLock::acquire(&temp_dir, "test_resource").expect("lock acquire failed");
            assert!(lock.lock_path.exists());
            let content = fs::read_to_string(&lock.lock_path).unwrap();
            assert_eq!(content.trim(), std::process::id().to_string());
        }
        // After drop, lock file must be cleaned up
        assert!(!temp_dir.join(".test_resource.lock").exists());

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_unpack_managed_archive_7z_success_and_fast_path() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_managed");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("node.exe"), b"mock node content").unwrap();
        fs::write(src_dir.join("readme.txt"), b"documentation").unwrap();

        let archive_path = temp_dir.join("node.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();
        assert!(archive_path.exists());

        let target_dir = temp_dir.join("node");
        // 1. Initial unpack
        let res = unpack_managed_archive_7z(&archive_path, &target_dir, "node.exe", false);
        assert!(res.is_ok(), "initial unpack failed: {:?}", res.err());
        assert!(target_dir.join("node.exe").exists());
        assert!(target_dir.join("readme.txt").exists());

        // 2. Fast-path check: calling again skips extraction instantly
        let res2 = unpack_managed_archive_7z(&archive_path, &target_dir, "node.exe", false);
        assert!(res2.is_ok(), "fast-path unpack failed: {:?}", res2.err());

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_unpack_managed_archive_7z_missing_expected_file_fails() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_unpack_missing_expected");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("other_file.txt"), b"not the expected file").unwrap();

        let archive_path = temp_dir.join("package.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();

        let target_dir = temp_dir.join("output");
        let res = unpack_managed_archive_7z(&archive_path, &target_dir, "expected_binary.exe", false);
        assert!(res.is_err(), "unpacking without expected file must fail");
        assert!(!target_dir.exists(), "target dir must not exist on failed validation");
        assert!(!target_dir.with_extension("staging").exists(), "staging dir must be wiped");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_unpack_real_scratch_test_archives() {
        let scratch_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("scratch");
        let node_7z = scratch_dir.join("node.test.7z");
        let backend_7z = scratch_dir.join("backend.test.7z");

        if node_7z.exists() {
            let temp_node = std::env::temp_dir().join("omnitome_test_unpack_real_node");
            let _ = wipe_dir(&temp_node);
            let res = unpack_managed_archive_7z(&node_7z, &temp_node, "node.exe", false);
            assert!(res.is_ok(), "failed to unpack scratch/node.test.7z: {:?}", res.err());
            assert!(temp_node.join("node.exe").exists(), "node.exe must exist in unpacked node archive");
            let _ = wipe_dir(&temp_node);
        }

        if backend_7z.exists() {
            let temp_backend = std::env::temp_dir().join("omnitome_test_unpack_real_backend");
            let _ = wipe_dir(&temp_backend);
            let res = unpack_managed_archive_7z(&backend_7z, &temp_backend, "server.js", false);
            assert!(res.is_ok(), "failed to unpack scratch/backend.test.7z: {:?}", res.err());
            assert!(temp_backend.join("server.js").exists(), "server.js must exist in unpacked backend archive");
            assert!(temp_backend.join("node_modules").is_dir(), "node_modules must exist in unpacked backend archive");
            let _ = wipe_dir(&temp_backend);
        }
    }

    #[test]
    fn test_validate_entry_name() {
        // Safe entries
        assert!(validate_entry_name("node.exe").is_ok());
        assert!(validate_entry_name("sub/dir/server.js").is_ok());
        assert!(validate_entry_name("node_modules/@tauri-apps/api/index.js").is_ok());

        // Path traversal attempts
        assert!(validate_entry_name("../evil.exe").is_err());
        assert!(validate_entry_name("..\\evil.exe").is_err());
        assert!(validate_entry_name("foo/../../bar.exe").is_err());
        assert!(validate_entry_name("a/b/../c").is_err());

        // Root path attempts
        assert!(validate_entry_name("/root.exe").is_err());
        assert!(validate_entry_name("\\root.exe").is_err());
        assert!(validate_entry_name("/etc/passwd").is_err());

        // Drive specifier attempts
        assert!(validate_entry_name(r"C:\Windows\System32\cmd.exe").is_err());
        assert!(validate_entry_name("d:malicious.dll").is_err());
        assert!(validate_entry_name("D:/games").is_err());
    }

    #[test]
    fn test_get_candidate_sidecar_archives_strict_order() {
        let install_dir = PathBuf::from(r"C:\Program Files\Omnitome");
        let archives = get_candidate_sidecar_archives(&install_dir);
        assert_eq!(archives.len(), 2);
        assert_eq!(archives[0], install_dir.join("resources").join("sidecar.7z"));
        assert_eq!(archives[1], install_dir.join("sidecar.7z"));

        // With verbatim prefix
        let verbatim_install = PathBuf::from(r"\\?\C:\Program Files\Omnitome");
        let verbatim_archives = get_candidate_sidecar_archives(&verbatim_install);
        assert_eq!(verbatim_archives.len(), 2);
        assert!(!verbatim_archives[0].to_string_lossy().starts_with(r"\\?\"));
        assert_eq!(verbatim_archives[0], install_dir.join("resources").join("sidecar.7z"));

        // Root and empty check
        assert!(get_candidate_sidecar_archives(Path::new("")).is_empty());
        assert!(get_candidate_sidecar_archives(Path::new(r"C:\")).is_empty());
    }

    #[test]
    fn test_tauri_conf_windows_and_nsis_configuration() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let conf_path = manifest_dir.join("tauri.conf.json");
        assert!(conf_path.exists(), "tauri.conf.json must exist");

        let content = fs::read_to_string(&conf_path).expect("failed to read tauri.conf.json");
        let v: serde_json::Value = serde_json::from_str(&content).expect("invalid json in tauri.conf.json");

        let windows = v["app"]["windows"].as_array().expect("app.windows should be an array");
        assert_eq!(windows.len(), 2, "must configure exactly 2 windows");

        let main_win = windows.iter().find(|w| w["label"] == "main").expect("main window missing");
        assert_eq!(main_win["url"], "index.html");
        assert_eq!(main_win["visible"], false, "main window must start hidden");

        let splash_win = windows.iter().find(|w| w["label"] == "splashscreen").expect("splashscreen window missing");
        assert_eq!(splash_win["url"], "splashscreen.html");
        assert_eq!(splash_win["width"], 450);
        assert_eq!(splash_win["height"], 300);
        assert_eq!(splash_win["center"], true);
        assert_eq!(splash_win["decorations"], false);
        assert_eq!(splash_win["alwaysOnTop"], true);
        assert_eq!(splash_win["visible"], true);
        assert_eq!(splash_win["resizable"], false);

        let nsis = &v["bundle"]["windows"]["nsis"];
        assert_eq!(nsis["installMode"], "currentUser");
        assert!(nsis.get("installerHooks").is_none(), "installerHooks must not be configured");
    }

    #[test]
    fn test_splashscreen_html_content() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let splash_path = manifest_dir.parent().unwrap().join("frontend").join("public").join("splashscreen.html");
        assert!(splash_path.exists(), "splashscreen.html must exist in frontend/public");

        let content = fs::read_to_string(&splash_path).expect("failed to read splashscreen.html");
        assert!(content.contains("Unpacking neural engine..."));
        assert!(content.contains("First launch may take 1-2 minutes."));
    }

    #[test]
    fn test_validate_entry_name_windows_dos_devices() {
        let reserved = [
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];

        for dev in &reserved {
            // Direct device name
            assert!(validate_entry_name(dev).is_err(), "Device {} should be rejected", dev);
            // Lowercase
            assert!(validate_entry_name(&dev.to_lowercase()).is_err(), "Lowercase {} should be rejected", dev);
            // With extension
            let with_ext = format!("{}.txt", dev);
            assert!(validate_entry_name(&with_ext).is_err(), "{} should be rejected", with_ext);
            let with_js = format!("{}.js", dev.to_lowercase());
            assert!(validate_entry_name(&with_js).is_err(), "{} should be rejected", with_js);
            // Nested in subdirectories
            let nested = format!("sub/dir/{}", dev);
            assert!(validate_entry_name(&nested).is_err(), "{} should be rejected", nested);
            let nested_ext = format!("node_modules/{}.exe", dev);
            assert!(validate_entry_name(&nested_ext).is_err(), "{} should be rejected", nested_ext);
        }

        // Permitted legitimate names that contain similar substrings
        assert!(validate_entry_name("contact.txt").is_ok());
        assert!(validate_entry_name("auxiliary.js").is_ok());
        assert!(validate_entry_name("sub/nursery.png").is_ok());
        assert!(validate_entry_name("computer.dll").is_ok());
    }

    #[test]
    fn test_validate_entry_name_ads_colons() {
        assert!(validate_entry_name("server.js:stream").is_err());
        assert!(validate_entry_name("file.txt::$DATA").is_err());
        assert!(validate_entry_name("sub/module.js:hidden").is_err());
        assert!(validate_entry_name("C:payload.exe").is_err());
        assert!(validate_entry_name(":colon_start").is_err());
    }

    #[test]
    fn test_validate_entry_name_trailing_dots_and_spaces() {
        assert!(validate_entry_name("file.txt.").is_err());
        assert!(validate_entry_name("file.txt ").is_err());
        assert!(validate_entry_name("sub./file.txt").is_err());
        assert!(validate_entry_name("sub /file.txt").is_err());
        assert!(validate_entry_name(" sub/file.txt").is_err());
        assert!(validate_entry_name("sub//file.txt").is_err());
        assert!(validate_entry_name("sub/./file.txt").is_err());
    }

    #[test]
    fn test_validate_entry_attributes_reparse_point_and_symlinks() {
        let mut entry = sevenz_rust::SevenZArchiveEntry::new();
        entry.name = "symlink_test".to_string();

        // 1. Normal file attributes pass
        entry.has_windows_attributes = true;
        entry.windows_attributes = 0x20; // FILE_ATTRIBUTE_ARCHIVE
        assert!(validate_entry_attributes(&entry).is_ok());

        // 2. Windows Reparse Point (0x400) rejected
        entry.windows_attributes = 0x00000420; // Reparse point + Archive
        let res = validate_entry_attributes(&entry);
        assert!(res.is_err(), "Reparse point must be rejected");
        assert!(res.unwrap_err().to_string().contains("reparse point"));

        // 3. POSIX symlink (0o120000 << 16 = 0xA0000000) rejected
        entry.windows_attributes = 0xA1ED0020;
        let res_posix = validate_entry_attributes(&entry);
        assert!(res_posix.is_err(), "POSIX symlink must be rejected");
        assert!(res_posix.unwrap_err().to_string().contains("POSIX symlink"));
    }

    #[test]
    fn test_compute_sha256_known_nist_vectors() {
        assert_eq!(
            compute_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            compute_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_archive_resource_limits_rejection() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_limits_rejection");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("f1.txt"), vec![b'A'; 2000]).unwrap();
        fs::write(src_dir.join("f2.txt"), vec![b'B'; 2000]).unwrap();
        fs::write(src_dir.join("f3.txt"), vec![b'C'; 2000]).unwrap();

        let archive_path = temp_dir.join("test_limits.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();

        // 1. Max entries rejection
        let tight_entries = ArchiveLimits {
            max_entries: 2,
            max_uncompressed_bytes: 100_000,
            max_single_file_bytes: 100_000,
        };
        let res_entries = validate_archive_entries(&archive_path, tight_entries);
        assert!(res_entries.is_err(), "Should reject when entry count exceeds limit");
        assert!(res_entries.unwrap_err().to_string().contains("exceeding maximum permitted limit of 2"));

        // 2. Max uncompressed size rejection
        let tight_bytes = ArchiveLimits {
            max_entries: 50,
            max_uncompressed_bytes: 3_000,
            max_single_file_bytes: 100_000,
        };
        let res_bytes = validate_archive_entries(&archive_path, tight_bytes);
        assert!(res_bytes.is_err(), "Should reject when uncompressed size exceeds limit");
        assert!(res_bytes.unwrap_err().to_string().contains("total uncompressed size exceeds"));

        // 3. Max single file size rejection
        let tight_single = ArchiveLimits {
            max_entries: 50,
            max_uncompressed_bytes: 100_000,
            max_single_file_bytes: 1_000,
        };
        let res_single = validate_archive_entries(&archive_path, tight_single);
        assert!(res_single.is_err(), "Should reject when single file exceeds limit");
        assert!(res_single.unwrap_err().to_string().contains("exceeding maximum single file limit"));

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_extraction_lock_timeout_and_concurrency() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_lock_timeout");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let lock1 = ExtractionLock::acquire_with_timeout(&temp_dir, "shared_res", Duration::from_secs(5))
            .expect("First acquire should succeed");

        // Concurrent acquire from a separate thread must time out (Win32 Mutex is re-entrant on same thread, exclusive across threads)
        let t_dir = temp_dir.clone();
        let handle = thread::spawn(move || {
            let start = std::time::Instant::now();
            let res = ExtractionLock::acquire_with_timeout(&t_dir, "shared_res", Duration::from_millis(150));
            (res.is_err(), start.elapsed())
        });

        let (timed_out, elapsed) = handle.join().unwrap();
        assert!(timed_out, "Concurrent acquire from another thread must time out");
        assert!(elapsed >= Duration::from_millis(100), "Should wait for timeout duration");

        // Drop first lock; subsequent acquire from another thread should succeed immediately
        drop(lock1);

        let t_dir2 = temp_dir.clone();
        let handle2 = thread::spawn(move || {
            ExtractionLock::acquire_with_timeout(&t_dir2, "shared_res", Duration::from_secs(2)).is_ok()
        });
        assert!(handle2.join().unwrap(), "Acquire after release should succeed");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    #[cfg(windows)]
    fn test_extraction_lock_abandoned_mutex_recovery() {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let temp_dir = std::env::temp_dir().join("omnitome_test_lock_abandoned");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let mutex_name = compute_mutex_name(&temp_dir, "test_abandon");
        let mutex_name_clone = mutex_name.clone();

        #[link(name = "kernel32")]
        extern "system" {
            fn ExitThread(dwExitCode: u32) -> !;
            fn CreateMutexW(lpMutexAttributes: *mut std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> *mut std::ffi::c_void;
            fn WaitForSingleObject(hHandle: *mut std::ffi::c_void, dwMilliseconds: u32) -> u32;
        }

        // Spawn a thread that acquires the raw Win32 mutex and exits abnormally without releasing it
        let _ = thread::spawn(move || {
            let name_wide: Vec<u16> = OsStr::new(&mutex_name_clone).encode_wide().chain(Some(0)).collect();
            let raw_mutex = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name_wide.as_ptr()) };
            assert!(!raw_mutex.is_null());
            let wait_res = unsafe { WaitForSingleObject(raw_mutex, 2000) };
            assert_eq!(wait_res, 0); // WAIT_OBJECT_0
            // Terminate thread abnormally without ReleaseMutex
            unsafe { ExitThread(0); }
        });

        // Give the OS thread time to terminate and abandon the mutex
        thread::sleep(Duration::from_millis(200));

        // Main thread acquires: Windows kernel must report WAIT_ABANDONED and award ownership safely
        let lock = ExtractionLock::acquire_with_timeout(&temp_dir, "test_abandon", Duration::from_secs(2))
            .expect("Should acquire abandoned mutex without deadlock or stale lock failure");
        assert!(lock.was_abandoned, "Should flag that the mutex was abandoned by dead owner");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_promote_staging_dir_success_and_rollback() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_promote_rollback");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("runtime");
        let staging_dir = temp_dir.join("runtime.staging");
        let backup_dir = temp_dir.join("runtime.backup");

        // 1. Initial promote
        fs::create_dir_all(&staging_dir).unwrap();
        fs::write(staging_dir.join("version.txt"), b"v1").unwrap();
        promote_staging_dir(&staging_dir, &target_dir).unwrap();
        assert!(target_dir.join("version.txt").exists());
        assert_eq!(fs::read_to_string(target_dir.join("version.txt")).unwrap(), "v1");
        assert!(!staging_dir.exists());
        assert!(!backup_dir.exists());

        // 2. Successful update promote
        fs::create_dir_all(&staging_dir).unwrap();
        fs::write(staging_dir.join("version.txt"), b"v2").unwrap();
        promote_staging_dir(&staging_dir, &target_dir).unwrap();
        assert_eq!(fs::read_to_string(target_dir.join("version.txt")).unwrap(), "v2");
        assert!(!backup_dir.exists());

        // 3. Rollback simulation: if staging cannot be renamed, target v2 must be preserved
        // We simulate this by having target exist, but staging_dir does NOT exist
        let non_existent_staging = temp_dir.join("runtime.staging_ghost");
        let promote_err = promote_staging_dir(&non_existent_staging, &target_dir);
        assert!(promote_err.is_err(), "Promoting non-existent staging must fail");
        // Target must remain intact with v2 content via rollback
        assert!(target_dir.join("version.txt").exists(), "Target must be restored after rollback");
        assert_eq!(fs::read_to_string(target_dir.join("version.txt")).unwrap(), "v2");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_startup_recovery_interrupted_rotations() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_startup_recovery");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("runtime");
        let backup_dir = temp_dir.join("runtime.backup");
        let staging_dir = temp_dir.join("runtime.staging");

        // Case A: target missing, backup has valid runtime, partial staging present
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("server.js"), b"working server v1").unwrap();
        fs::create_dir_all(&staging_dir).unwrap();
        fs::write(staging_dir.join("partial.tmp"), b"incomplete").unwrap();

        recover_interrupted_rotations(&target_dir, "server.js");

        assert!(target_dir.join("server.js").exists(), "Target directory must be restored from backup");
        assert_eq!(fs::read_to_string(target_dir.join("server.js")).unwrap(), "working server v1");
        assert!(!backup_dir.exists(), "Backup directory should have been rotated to target");
        assert!(!staging_dir.exists(), "Partial staging directory must be swept");

        // Case B: target exists, leftover backup present
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("old.txt"), b"stale").unwrap();

        recover_interrupted_rotations(&target_dir, "server.js");

        assert!(target_dir.join("server.js").exists(), "Target directory must remain intact");
        assert!(!backup_dir.exists(), "Leftover backup directory must be wiped");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_manifest_generation_update_detection_and_stub_rejection() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_manifest_lifecycle");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. Initial extraction of v1 archive
        let src_v1 = temp_dir.join("src_v1");
        fs::create_dir_all(&src_v1).unwrap();
        fs::write(src_v1.join("node.exe"), b"node binary version 1.0").unwrap();
        fs::write(src_v1.join("package.json"), b"{\"name\": \"omnitome\"}").unwrap();

        let archive_v1 = temp_dir.join("node_v1.7z");
        sevenz_rust::compress_to_path(&src_v1, &archive_v1).unwrap();

        let target_dir = temp_dir.join("node");
        unpack_managed_archive_7z(&archive_v1, &target_dir, "node.exe", false).expect("Unpack v1 failed");

        // Manifest must exist and be valid
        let manifest_path = target_dir.join(".extraction_manifest.json");
        assert!(manifest_path.exists(), "Manifest must be written upon extraction");
        assert!(is_manifest_valid(&target_dir, &archive_v1, "node.exe"));

        // Fast path skips instantly
        assert!(unpack_managed_archive_7z(&archive_v1, &target_dir, "node.exe", false).is_ok());

        // 2. Update detection: archive changes to v2
        let src_v2 = temp_dir.join("src_v2");
        fs::create_dir_all(&src_v2).unwrap();
        fs::write(src_v2.join("node.exe"), b"node binary version 2.0 UPDATED").unwrap();
        fs::write(src_v2.join("package.json"), b"{\"name\": \"omnitome-v2\"}").unwrap();

        let archive_v2 = temp_dir.join("node_v2.7z");
        sevenz_rust::compress_to_path(&src_v2, &archive_v2).unwrap();

        // Calling unpack with updated archive must detect hash mismatch and promote v2
        assert!(!is_manifest_valid(&target_dir, &archive_v2, "node.exe"), "Must detect archive hash mismatch");
        unpack_managed_archive_7z(&archive_v2, &target_dir, "node.exe", false).expect("Unpack v2 failed");

        assert_eq!(
            fs::read_to_string(target_dir.join("node.exe")).unwrap(),
            "node binary version 2.0 UPDATED",
            "Target runtime must be updated to v2"
        );
        assert!(is_manifest_valid(&target_dir, &archive_v2, "node.exe"));

        // 3. Stub file rejection: truncate critical binary to 0 bytes
        fs::write(target_dir.join("node.exe"), b"").unwrap();
        assert_eq!(fs::metadata(target_dir.join("node.exe")).unwrap().len(), 0);
        assert!(!is_manifest_valid(&target_dir, &archive_v2, "node.exe"), "0-byte stub must invalidate manifest");

        // Unpack heals the truncated binary
        unpack_managed_archive_7z(&archive_v2, &target_dir, "node.exe", false).expect("Re-unpack failed");
        assert!(fs::metadata(target_dir.join("node.exe")).unwrap().len() > 0, "Binary must be rehealed");

        // 4. Corrupted JSON manifest rejection
        fs::write(&manifest_path, b"{ broken json").unwrap();
        assert!(!is_manifest_valid(&target_dir, &archive_v2, "node.exe"), "Corrupted JSON must invalidate manifest");

        // Unpack heals the corrupted manifest
        unpack_managed_archive_7z(&archive_v2, &target_dir, "node.exe", false).expect("Re-unpack failed");
        assert!(is_manifest_valid(&target_dir, &archive_v2, "node.exe"), "Manifest must be regenerated");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_validate_production_and_scratch_archives_unmodified() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest_dir.parent().unwrap();
        let scratch_dir = root.join("scratch");
        let resources_dir = root.join("resources");

        let node_7z = scratch_dir.join("node.test.7z");
        let backend_7z = scratch_dir.join("backend.test.7z");
        let sidecar_7z = resources_dir.join("sidecar.7z");

        // Validate scratch node archive
        if node_7z.exists() {
            let meta_before = fs::metadata(&node_7z).unwrap();
            let res = validate_archive_entries(&node_7z, ArchiveLimits::node());
            assert!(res.is_ok(), "node.test.7z validation failed: {:?}", res.err());
            let meta_after = fs::metadata(&node_7z).unwrap();
            assert_eq!(meta_before.len(), meta_after.len(), "node.test.7z size modified!");
        }

        // Validate scratch backend archive
        if backend_7z.exists() {
            let meta_before = fs::metadata(&backend_7z).unwrap();
            let res = validate_archive_entries(&backend_7z, ArchiveLimits::backend());
            assert!(res.is_ok(), "backend.test.7z validation failed: {:?}", res.err());
            let meta_after = fs::metadata(&backend_7z).unwrap();
            assert_eq!(meta_before.len(), meta_after.len(), "backend.test.7z size modified!");
        }

        // Validate production sidecar archive structure and limits without modifying it
        if sidecar_7z.exists() {
            let meta_before = fs::metadata(&sidecar_7z).unwrap();
            let res = validate_archive_entries(&sidecar_7z, ArchiveLimits::sidecar());
            assert!(res.is_ok(), "resources/sidecar.7z validation failed: {:?}", res.err());
            let meta_after = fs::metadata(&sidecar_7z).unwrap();
            assert_eq!(meta_before.len(), meta_after.len(), "sidecar.7z size modified!");
        }
    }

    #[test]
    fn test_recover_interrupted_rotations_preserves_valid_backup_when_target_is_empty_or_broken() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_recovery_broken_target");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("runtime");
        let backup_dir = temp_dir.join("runtime.backup");
        let staging_dir = temp_dir.join("runtime.staging");

        // Simulate interrupted promotion where target exists but has missing or broken binary,
        // and backup_dir holds the last valid runtime.
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("corrupted.tmp"), b"partial broken data").unwrap();

        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("node.exe"), b"valid working node binary").unwrap();

        fs::create_dir_all(&staging_dir).unwrap();
        fs::write(staging_dir.join("partial.7z"), b"partial").unwrap();

        recover_interrupted_rotations(&target_dir, "node.exe");

        // The supervisor MUST NOT wipe backup; it must restore backup -> target!
        assert!(target_dir.join("node.exe").exists(), "Valid runtime must be restored to target");
        assert_eq!(
            fs::read_to_string(target_dir.join("node.exe")).unwrap(),
            "valid working node binary",
            "Restored runtime content must match the backup"
        );
        assert!(!backup_dir.exists(), "Backup directory should have been rotated to target");
        assert!(!staging_dir.exists(), "Leftover staging artifacts must be purged");

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_extraction_lock_case_and_slash_normalization() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_lock_case_norm");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Path representation 1: standard backslashes
        let p1 = temp_dir.clone();
        // Path representation 2: forward slashes, upper/lowercase variations
        let p2_str = temp_dir.to_string_lossy().replace('\\', "/").to_uppercase();
        let p2 = PathBuf::from(p2_str);

        let lock1 = ExtractionLock::acquire_with_timeout(&p1, "norm_test", Duration::from_secs(5))
            .expect("First acquire with path 1 should succeed");

        // Concurrent acquire on p2 (different casing and slashes, same folder) from another thread MUST time out
        let handle = thread::spawn(move || {
            ExtractionLock::acquire_with_timeout(&p2, "NORM_TEST", Duration::from_millis(150)).is_err()
        });

        let timed_out = handle.join().unwrap();
        assert!(timed_out, "Lock on normalized path variation must be recognized as held and time out");

        drop(lock1);
        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_validate_entry_name_forbidden_windows_chars_and_clock() {
        // Forbidden Win32 characters: < > " | ? * and control chars
        let forbidden = ["file<1>.txt", "pipe|cmd.js", "wild*card.dll", "test?mark.dat", "quote\"file.json"];
        for name in &forbidden {
            assert!(validate_entry_name(name).is_err(), "{:?} should be rejected", name);
        }

        // Control characters
        assert!(validate_entry_name("bad\x01file.txt").is_err());
        assert!(validate_entry_name("bad\x1Ffile.txt").is_err());

        // CLOCK$ device name
        assert!(validate_entry_name("CLOCK$").is_err());
        assert!(validate_entry_name("clock$").is_err());
        assert!(validate_entry_name("clock$.txt").is_err());
        assert!(validate_entry_name("sub/dir/clock$").is_err());
    }

    #[test]
    fn test_manifest_rejects_archive_name_mismatch() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_manifest_archive_mismatch");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("node.exe"), b"mock node").unwrap();

        let archive_node = temp_dir.join("node.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_node).unwrap();

        let target_dir = temp_dir.join("node");
        unpack_managed_archive_7z(&archive_node, &target_dir, "node.exe", false).expect("Unpack node failed");
        assert!(is_manifest_valid(&target_dir, &archive_node, "node.exe"));

        // If someone passes a different archive path (e.g. backend.7z) against this target directory,
        // it must reject the fast-path due to archive name mismatch
        let archive_backend = temp_dir.join("backend.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_backend).unwrap();

        assert!(
            !is_manifest_valid(&target_dir, &archive_backend, "node.exe"),
            "Manifest created for node.7z must NOT validate for backend.7z"
        );

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_decompression_in_flight_stream_size_limit_rejection() {
        let temp_dir = std::env::temp_dir().join("omnitome_test_stream_limits");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let src_dir = temp_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("large_file.dat"), vec![b'X'; 50_000]).unwrap();

        let archive_path = temp_dir.join("payload.7z");
        sevenz_rust::compress_to_path(&src_dir, &archive_path).unwrap();

        let tight_single_limit = ArchiveLimits {
            max_entries: 100,
            max_uncompressed_bytes: 1_000_000,
            max_single_file_bytes: 10_000, // 10 KB limit vs 50 KB file
        };

        // Header check catches declared size
        assert!(validate_archive_entries(&archive_path, tight_single_limit).is_err());

        let _ = wipe_dir(&temp_dir);
    }

    #[test]
    fn test_extraction_lock_cross_process_mutual_exclusion() {
        if std::env::var("OMNITOME_TEST_LOCK_CHILD").is_ok() {
            let lock_dir = PathBuf::from(std::env::var("OMNITOME_CHILD_LOCK_DIR").unwrap());
            let timeout_ms: u64 = std::env::var("OMNITOME_CHILD_TIMEOUT_MS").unwrap().parse().unwrap();
            let res = ExtractionLock::acquire_with_timeout(&lock_dir, "proc_lock", Duration::from_millis(timeout_ms));
            if res.is_ok() {
                std::process::exit(0);
            } else {
                std::process::exit(42);
            }
        }

        let temp_dir = std::env::temp_dir().join("omnitome_test_lock_cross_proc");
        let _ = wipe_dir(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let lock1 = ExtractionLock::acquire_with_timeout(&temp_dir, "proc_lock", Duration::from_secs(5))
            .expect("Parent should acquire lock");

        let current_exe = std::env::current_exe().expect("current_exe failed");

        // Spawn child process while parent holds lock (short timeout 150ms -> must fail with code 42)
        let status1 = Command::new(&current_exe)
            .args(["--exact", "tests::test_extraction_lock_cross_process_mutual_exclusion"])
            .env("OMNITOME_TEST_LOCK_CHILD", "1")
            .env("OMNITOME_CHILD_LOCK_DIR", temp_dir.to_string_lossy().to_string())
            .env("OMNITOME_CHILD_TIMEOUT_MS", "150")
            .status()
            .expect("Failed to execute child process");
        assert_eq!(status1.code(), Some(42), "Child process should fail to acquire lock while parent holds it");

        // Drop parent lock; child process with longer timeout must succeed (code 0)
        drop(lock1);

        let status2 = Command::new(&current_exe)
            .args(["--exact", "tests::test_extraction_lock_cross_process_mutual_exclusion"])
            .env("OMNITOME_TEST_LOCK_CHILD", "1")
            .env("OMNITOME_CHILD_LOCK_DIR", temp_dir.to_string_lossy().to_string())
            .env("OMNITOME_CHILD_TIMEOUT_MS", "3000")
            .status()
            .expect("Failed to execute child process");
        assert_eq!(status2.code(), Some(0), "Child process should acquire lock after parent released it");

        let _ = wipe_dir(&temp_dir);
    }
}
