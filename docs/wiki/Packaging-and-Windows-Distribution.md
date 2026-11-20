# Packaging & Windows Distribution

Building and distributing a local-first neural desktop application on Windows introduces significant binary distribution and OS-level packaging hurdles. 

Omnitome bundles a complete Node.js 22 LTS runtime, a PyTorch FP16 CUDA runtime with cuDNN acceleration libraries, and local model weights into a self-contained native Windows executable. This document details how Omnitome overcomes the hard limits of desktop packaging on Windows.

---

## 1. The 32-Bit NSIS 2.0 GB Ceiling

### The Problem
When packaging a desktop application containing PyTorch, CUDA runtime libraries, NVIDIA cuDNN kernels, spaCy, and Python binaries:
- The uncompressed Python sidecar directory totals **~4.6 GB**.
- Most desktop installer frameworks on Windows, including the standard NSIS (Nullsoft Scriptable Install System) bundled with Tauri, are 32-bit binaries.
- The 32-bit architecture enforces a hard **2.048 GB (2,147,483,647 bytes) payload limit** on the generated `.exe` installer. Any attempt to bundle the uncompressed sidecar results in immediate installer compilation failure or corrupt installations.

```
+-------------------------------------------------------------------------------+
|                       THE NSIS 2.0 GB DISTRIBUTION CLIFF                      |
+-------------------------------------------------------------------------------+
| Uncompressed Sidecar: [ 4.6 GB ] ──> NSIS 32-bit Limit [ 2.0 GB ] ──> CRASH   |
| Solution: LZMA .7z:   [ 1.36 GB ] ──> Bundled into Installer ───────> SUCCESS |
+-------------------------------------------------------------------------------+
```

### The Resolution: High-Ratio LZMA 7z Extraction
Omnitome solves the payload ceiling using a two-stage extraction architecture:
1. **LZMA Compression:** The frozen Python sidecar is packaged into `resources/sidecar.7z` with maximum LZMA compression, shrinking the payload to **~1.36 GB** (well below the 2 GB threshold).
2. **First-Launch Unpacking:** The Tauri v2 desktop shell embeds `sevenz-rust`. On the user's initial launch, the Rust supervisor detects if the sidecar is unpacked. If absent, it decompresses `sidecar.7z` directly into:
   ```
   %LOCALAPPDATA%\Omnitome\sidecar\
   ```
3. **Integrity Validation:** Extraction progress is reported to the user interface via native Tauri events. Subsequent launches bypass extraction entirely, executing the Python sidecar directly from LocalAppData.

---

## 2. PyInstaller Freezing & Offline Dependencies

Freezing deep learning frameworks with PyInstaller frequently leads to runtime missing DLL errors or silent fallbacks to CPU mode. Omnitome configures `sidecars/python/sidecar.spec` to guarantee 100% offline functionality:

### 2.1 Dynamic CUDA & cuDNN DLL Harvesting
Modern PyTorch wheels install CUDA dependencies across multiple isolated Python packages (`nvidia.cudnn`, `nvidia.cublas`, `nvidia.cufft`). PyInstaller does not automatically detect these dynamic links:
- `sidecar.spec` uses `collect_all('torch')`.
- It executes a PEP 420 namespace scan across all `nvidia.*` package directories, copying every `*.dll` into the root `_internal/` directory. This ensures that cuDNN engines (e.g., `cudnn_engines_precompiled64_9.dll`) are discoverable at load time.

### 2.2 Pre-Bundled Phonetics & Language Models
Kokoro-82M and its phonemizer dependencies (`misaki`, `espeakng_loader`) often attempt to download language files on first run:
- Omnitome pre-bundles `en_core_web_sm` (spaCy) directly into the sidecar.
- It bundles phonetic dictionaries via `collect_data_files('espeakng_loader')`.
- `sidecar.spec` ensures that startup initialization takes **< 45 ms** without network egress.

### 2.3 Strict Offline Voice Resolution
When `HF_HUB_OFFLINE=1` is enforced, default Hugging Face download methods throw `OfflineModeIsEnabled`. Omnitome implements `_resolve_voice_path`:
- Probes local application directories (`models/kokoro/`), local resource folders, and cached snapshot directories.
- Loads voice tensors (`.pt`) directly from disk, preventing network calls.

---

## 3. Windows Verbatim Path Prefix Sanitization (`\\?\`)

### The Hazard
Windows APIs and Tauri's Rust filesystem helpers frequently resolve absolute canonical paths prefixed with the Windows verbatim UNC prefix:
```
\\?\C:\Users\Username\AppData\Local\Omnitome\...
```

When verbatim paths are passed to Node.js 22 LTS filesystem methods (`fs.readdir`, `fs.lstat`, `fs.watch`), Node's libuv implementation encounters path parsing ambiguities on Windows drive roots, resulting in fatal crashes:
```
EISDIR: illegal operation on a directory, lstat 'C:'
```

### The Resolution
Omnitome implements mandatory UNC path sanitization at all process boundaries:
```rust
// src-tauri/src/utils.rs
pub fn strip_windows_verbatim(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with(r"\\?\") {
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}
```
All paths crossing from Tauri/Rust into Node.js or Python are passed through this normalizer.

---

## 4. WebView2 Audio Autoplay & Gapless Playback

### The WebView2 Autoplay Policy
Windows WebView2 implements strict autoplay security policies: any asynchronous attempt to start an `AudioContext` without an immediate, synchronous user gesture (such as clicking a button) is blocked or muted by the browser engine.

### Shared AudioContext Architecture
Omnitome unifies all audio interactions under a shared, pre-unlocked `AudioContext` in `frontend/src/audio/soundPlayer.ts`:
- A single user gesture unlocks the shared context on first interaction.
- Both procedural sound effects (90 UI stems) and Kokoro neural speech streams share this unlocked node.

### Citation-Only Silence WAV Buffers
When synthesized text contains only citation backlinks (such as `[p12]`), the text-cleaning regex strips the citation markers, leaving an empty string `""`. Passing an empty string to Kokoro TTS generates an HTTP `400 Bad Request` exception, interrupting continuous playback.

**The Fix:** When the text cleaner detects an empty sentence, the Python sidecar returns a pre-synthesized **250ms mono 24 kHz 16-bit PCM WAV containing absolute silence**. This placeholder satisfies the player pipeline and preserves gapless narration without pronunciation glitches.
