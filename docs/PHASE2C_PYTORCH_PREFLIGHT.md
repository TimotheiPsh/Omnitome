# Phase 2C Pre-Flight Security & Architectural Investigation: PyTorch Upgrade

> **Document Status:** COMPLETED INVESTIGATION (NO IMPLEMENTATION COMMENCED)  
> **Target Sub-Phase:** Phase 2C (PyTorch CVE-2026-24747 & Checkpoint Integrity)  
> **Date:** November 20, 2026  
> **Branch:** `security/phase-2-dependencies`  
> **Author:** Shiori (Omnitome Co-Architect & Security Engineering)  
> **Target Audience:** Onii-sama (Ayush / Ayumu), Repository Maintainers, Security Auditors  
> **Execution Status:** Pre-flight investigation only. Zero package installations, zero model alterations, and zero remote pushes have been made.

---

## 1. Executive Summary

This report delivers the pre-flight architectural and security investigation for **Phase 2C**, focusing on the remediation of **CVE-2026-24747** (**GHSA-63cw-57p8-fm3p**) in PyTorch.

Our forensic investigation confirms:
1. **The Vulnerability is Real and Directly Exposed:** The claim that `weights_only=True` renders CVE-2026-24747 unreachable is **false**. The vulnerability resides inside the custom `weights_only` unpickler itself (`SETITEM`/`SETITEMS` opcode processing and storage header validation). Our Kokoro-82M speech synthesis engine directly invokes `torch.load(..., weights_only=True)` to deserialize `.pth` and `.pt` checkpoint files.
2. **Official Patched Wheels Exist for Our Environment:** PyTorch `2.10.0+cu126` (with CUDA 12.6 runtime bundled) is officially available on `https://download.pytorch.org/whl/cu126` for Windows x64 and Python 3.10 (`cp310-win_amd64`).
3. **Hardware & Driver Compatibility:** The host workstation's NVIDIA Driver (`610.88`, CUDA UMD 13.3) and RTX 3050 Laptop GPU (Ampere sm_86, 4GB VRAM) natively support CUDA 12.6 without host driver upgrades.
4. **Kokoro TTS Compatibility:** `kokoro==0.9.4` declares unconstrained `torch` dependencies and relies exclusively on standard tensor operations (`torch.stft`, `torch.istft`, `torch.hann_window`, `torch.load`) that remain fully supported and backward-compatible in PyTorch 2.10+.
5. **Packaging Budget Headroom:** The uncompressed PyTorch 2.10.0+cu126 wheel is ~175 MB larger than 2.6.0+cu124. Under LZMA ultra compression, `resources/sidecar.7z` is projected to increase from **1.600 GB to ~1.66 GB**, preserving ~340 MB of safe headroom below the NSIS 2.047 GB installer ceiling.
6. **Defense-in-Depth Cryptographic Verification:** We have designed a pre-load SHA-256 verification gate grounded in upstream Git LFS commit pointers, protecting against checkpoint tampering independently of PyTorch unpickler security.

---

## 2. Vulnerability & Upstream Patch Evidence

### CVE-2026-24747 / GHSA-63cw-57p8-fm3p
- **Vulnerability Title:** Remote Code Execution via Heap Memory Corruption in `torch.load(..., weights_only=True)`.
- **Advisory URL:** [GHSA-63cw-57p8-fm3p](https://github.com/pytorch/pytorch/security/advisories/GHSA-63cw-57p8-fm3p)
- **Vulnerability Type:** CWE-502 (Deserialization of Untrusted Data) leading to heap memory corruption.
- **Affected Releases:** `torch <= 2.9.1`.
- **Fixed Releases:** `torch >= 2.10.0`.
- **Mechanism:** In PyTorch 2.6.0 through 2.9.1, `weights_only=True` was designed as a safe alternative to Python's native `pickle`. However, the restricted unpickler state-machine failed to properly validate the container types receiving `SETITEM` and `SETITEMS` opcodes. An adversary supplying a crafted pickle payload with malformed object storage offsets can induce arbitrary heap writes and hijack instruction pointers during deserialization.
- **Exposure in Omnitome:** `kokoro` (`KModel.__init__`) and `sidecars/python/tts/synthesize.py` load `.pth` and `.pt` files using `torch.load(..., weights_only=True)`. In PyTorch 2.6.0+cu124, these loading paths are directly susceptible if the underlying weight files are substituted or tampered with.

---

## 3. Exact Environment Audit

The host workstation environment was inspected via Python runtime introspection and `nvidia-smi`:

| Dimension | Measured Value | Analysis / Compliance |
| :--- | :--- | :--- |
| **Operating System** | Windows 11 Home Single Language x64 (Build 26100) | Fully supported |
| **Python Version** | `3.10.11` (tags/v3.10.11:7d4cc5a, MSC v.1929 64 bit) | ABI Tag: `cp310-cp310-win_amd64` |
| **Current PyTorch** | `2.6.0+cu124` | Affected by CVE-2026-24747 |
| **Current Bundled CUDA** | `12.4` | Bundled in `torch/lib/` |
| **Workstation GPU** | NVIDIA GeForce RTX 3050 Laptop GPU | 4095.5 MB Total VRAM (4 GB) |
| **Compute Capability** | `(8, 6)` (NVIDIA Ampere Architecture) | Supported by CUDA 11.x, 12.x, 13.x |
| **NVIDIA Host Driver** | `610.88` (Driver Model: WDDM) | Supports CUDA UMD up to `13.3` |
| **Auxiliary Vision/Audio** | `torchvision` and `torchaudio` are **NOT** installed | Zero auxiliary package friction |

---

## 4. Official PyTorch Wheel & CUDA Availability

We queried the official PyTorch wheel distribution indices (`https://download.pytorch.org/whl/`):

### 1. PyTorch CUDA Index Query Results
- **Index `cu124` (`https://download.pytorch.org/whl/cu124`):**  
  PyTorch discontinued CUDA 12.4 builds after release `2.6.0+cu124`. No `2.10.0+cu124` wheel exists.
- **Index `cu126` (`https://download.pytorch.org/whl/cu126`):**  
  **`torch-2.10.0+cu126-cp310-cp310-win_amd64.whl`** is officially published and verified.  
  - Download URL: `https://download-r2.pytorch.org/whl/cu126/torch-2.10.0%2Bcu126-cp310-cp310-win_amd64.whl`  
  - Wheel Payload Size: **2,589.8 MB** (2.529 GB).
- **Index `cu128` (`https://download.pytorch.org/whl/cu128`):**  
  `torch-2.10.0+cu128-cp310-cp310-win_amd64.whl` is also published.

### 2. CUDA Compatibility & Driver Requirements
- **CUDA 12.6 Minimum Driver on Windows:** Requires NVIDIA driver version **>= 560.70**.
- **Host Workstation Driver:** NVIDIA Driver is **`610.88`**, far exceeding the minimum requirement.
- **CUDA Toolkit Requirement:** PyTorch binary wheels for Windows bundle all necessary dynamic libraries (`cublas64_12.dll`, `cublasLt64_12.dll`, `cudnn_ops64_9.dll`, `torch_cuda.dll`). The host system does **not** require a local CUDA Toolkit installation.
- **Compute Architecture Compatibility:** RTX 3050 has compute capability `sm_86` (Ampere). PyTorch `cu126` wheels include native pre-compiled SASS binaries for `sm_50, sm_60, sm_70, sm_75, sm_80, sm_86, sm_90`, ensuring zero JIT kernel compilation latency during startup.

---

## 5. Repository Checkpoint-Loading Inventory

A full static and dynamic audit of model-loading entry points was conducted across the active codebase:

| Location | Invoked API | Checkpoint Path & Format | Source & Provenance | Modifiability Post-Install | Startup Behavior |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `sidecars/python/tts/synthesize.py:180` | `KModel(..., model=model_path)` (delegates to `torch.load(..., weights_only=True)`) | `kokoro-v1_0.pth` (312.05 MB, PyTorch pickled state dict) | Bundled in `sidecars/python/models/kokoro/` & `%LOCALAPPDATA%\Omnitome\models\kokoro\` | Modifiable by local malware running in user session | Lazy-loaded on first TTS request (`is_model_installed()`) |
| `sidecars/python/tts/synthesize.py:213` | `torch.load(v_path, weights_only=True)` | `voices/{voice}.pt` (~512 KB each, PyTorch pickled tensor) | Bundled in `voices/` (`af_heart.pt`, `am_adam.pt`, `bf_alice.pt`, `bm_george.pt`) | Modifiable by local user/malware | Loaded during `load_engine()` warmup |
| `sidecars/python/embed/routes.py:73` | `SentenceTransformer(model_id_or_path, local_files_only=True)` | `pytorch_model.bin` (2,165.93 MB, PyTorch pickled weights) | Installed in `%LOCALAPPDATA%\Omnitome\models\sentence_transformers\models--BAAI--bge-m3\` | Modifiable by local user/malware | Warmed at application startup (`_load_embedder()`) |
| `sidecars/python/nlp/routes.py:69` | `CrossEncoder(model_id_or_path, local_files_only=True)` | `model.safetensors` (2,165.86 MB, **Safetensors**) | Installed in `%LOCALAPPDATA%\Omnitome\models\sentence_transformers\models--BAAI--bge-reranker-v2-m3\` | Modifiable by local user/malware | Lazy-loaded on first rerank request |
| `dist-sidecar` & `src-tauri/target/` | N/A (Build artifacts only) | Various internal references | Generated during packaging | N/A | Build staging only |

### Key Observations
1. **Safetensors vs Pickle:**  
   - The **BGE Reranker v2** is already utilizing **`model.safetensors`**, making it completely immune to pickle deserialization vulnerabilities.
   - **BGE-M3** currently uses `pytorch_model.bin` in our local AppData cache, though BGE-M3 also supports `model.safetensors`.
   - **Kokoro-82M** currently relies on `.pth` and `.pt` files.

---

## 6. Kokoro-82M Compatibility Assessment

### 1. Declared Dependencies
`pip show kokoro` and `METADATA` inspection:
```text
Requires-Dist: huggingface-hub
Requires-Dist: loguru
Requires-Dist: misaki[en]>=0.9.4
Requires-Dist: numpy
Requires-Dist: torch
Requires-Dist: transformers
```
Kokoro declares an unconstrained `torch` dependency with no upper version bound.

### 2. API Usage Audit
An audit of all 45 `torch.*` calls in `kokoro` (`KModel`, `KPipeline`, `istftnet`) reveals:
- Tensor creation and manipulation: `torch.zeros`, `torch.ones`, `torch.arange`, `torch.cat`, `torch.repeat_interleave`, `torch.sigmoid`.
- Spectral signal processing: `torch.stft`, `torch.istft`, `torch.hann_window`.
- Module architecture: `torch.nn.Module`, `torch.nn.Linear`.
- Deserialization: `torch.load(..., map_location='cpu', weights_only=True)`.

None of these APIs have been deprecated or modified in PyTorch 2.10+. Kokoro's native 24kHz synthesis logic and ISTFTNet spectrogram decoder remain completely intact.

---

## 7. Cryptographic Checkpoint Integrity Architecture

Relying solely on runtime framework patches is insufficient for high-assurance desktop applications. We propose establishing an independent **Cryptographic Checkpoint Integrity Gate**.

```text
┌────────────────────────────────────────────────────────┐
│  Upstream Git LFS Pointer (Hugging Face Model Repo)    │
│  e.g., hexgrad/Kokoro-82M/kokoro-v1_0.pth              │
│  oid sha256:496dba118d1a58f5f3db2efc88dbdc216e...     │
└───────────────────────────┬────────────────────────────┘
                            │ (Extracted at build/spec time)
                            ▼
┌────────────────────────────────────────────────────────┐
│  Trusted Read-Only Checksum Manifest                   │
│  Embedded in sidecar / resources/models/manifest.json  │
└───────────────────────────┬────────────────────────────┘
                            │ Pre-load SHA-256 verification
                            ▼
┌────────────────────────────────────────────────────────┐
│  Target Model Weight on Disk (%LOCALAPPDATA%)          │
│  Streaming SHA-256 Check (64KB chunks)                 │
└───────────────────────────┬────────────────────────────┘
               │ Match?
        ┌──────┴──────┐
        ▼             ▼
      [ YES ]       [ NO ] ──► FAIL CLOSED: Raise IntegrityError
        │                     (Never invoke torch.load)
        ▼
   torch.load()
```

### 1. Independent Provenance of Hashes
Hashes **MUST NOT** be computed from local disk files and blindly trusted. Instead, expected SHA-256 hashes must be sourced directly from the upstream repository's official Git LFS pointers:

*Verified Upstream LFS Pointer for Kokoro-82M:*
```text
URL: https://huggingface.co/hexgrad/Kokoro-82M/raw/main/kokoro-v1_0.pth
version https://git-lfs.github.com/spec/v1
oid sha256:496dba118d1a58f5f3db2efc88dbdc216e0483fc89fe6e47ee1f2c53f18ad1e4
size 327212226
```
*Measured Local File:*
```text
kokoro-v1_0.pth SHA-256: 496dba118d1a58f5f3db2efc88dbdc216e0483fc89fe6e47ee1f2c53f18ad1e4 (Exact match)
```

### 2. Manifest Storage & Protection
- The manifest (`checksums.json`) will be stored inside the read-only, signed application bundle directory (`resources/models/checksums.json` or compiled directly into Python bytecode), rather than alongside the writable cache in `%LOCALAPPDATA%`.
- If a user or malware modifies weights in `%LOCALAPPDATA%`, the application detects the checksum mismatch against the protected manifest and **fails closed**.

### 3. Fail-Closed Protocol
Before `torch.load()` or `SentenceTransformer()` is executed:
1. Stream file bytes through `hashlib.sha256()` in 64 KB blocks.
2. If the file is not in the manifest: **FAIL CLOSED** (`UnknownModelFileError`).
3. If the computed hash differs from the manifest: **FAIL CLOSED** (`ModelIntegrityError`). Do not invoke `torch.load()`.
4. Surface a clear, human-readable UI error: *"Model file failed integrity verification. Please reinstall model weights from a trusted source."*

### 4. Scope: What Cryptographic Verification Protects Against
- **Protects Against:** Checkpoint file tampering, trojan weight substitution, partial/corrupted downloads, disk bit-rot, and untrusted model injections.
- **Does NOT Protect Against:** Post-loading runtime memory manipulation (e.g. process injection via admin/debugger), GPU adversarial inputs, or logic bugs inside the model graph itself.

---

## 8. Packaging & NSIS Size Constraints

The Windows installer is constrained by the 32-bit NSIS payload ceiling of **2.047 GB** (`2,147,483,647` bytes). Exceeding this limit causes `makensis` to crash catastrophically.

### 1. Current Baseline Measurements
- `resources/sidecar.7z`: **1,638.22 MB** (**1.600 GB**).
- Available NSIS Headroom: **~409 MB**.
- Uncompressed `sidecars/python/.venv`: **6,415.44 MB** (6.265 GB).
- Largest existing DLLs in `.venv`:
  - `torch_cuda.dll`: `912.92 MB`
  - `cudnn_engines_precompiled64_9.dll`: `561.63 MB`
  - `cublasLt64_12.dll`: `450.90 MB`
  - `cufft64_11.dll`: `278.27 MB`
  - `cusparse64_12.dll`: `262.86 MB`

### 2. Impact of PyTorch 2.10.0+cu126 Wheel
- `torch-2.6.0+cu124` wheel size: **2,415.0 MB**.
- `torch-2.10.0+cu126` wheel size: **2,589.8 MB**.
- Wheel size increase: **+174.8 MB** (Estimated uncompressed disk delta: `~350 MB`).
- **LZMA Compression Ratio:** CUDA DLLs and PyTorch runtime binaries compress at a ratio of approximately 4.5:1 to 5:1. An uncompressed delta of `~350 MB` yields an estimated compressed archive delta of **+50 MB to +70 MB**.
- **Projected `sidecar.7z` Size:** **~1,690 MB to ~1,710 MB** (**1.65 GB to 1.67 GB**).
- **Projected Headroom:** **~340 MB below the NSIS ceiling**. Safe to package.

---

## 9. Recommended Target Version & Implementation Strategy

### Recommended Version Pin
```text
--extra-index-url https://download.pytorch.org/whl/cu126
torch==2.10.0+cu126
```

### Implementation Safeguards (Upon Authorization)
1. **Branch Isolation:** Retain all work on `security/phase-2-dependencies`.
2. **Rollback Snapshot:** Take a pip freeze snapshot and archive current `.venv/Lib/site-packages/torch` metadata prior to installation.
3. **Spec File Update:** Inspect `sidecars/python/sidecar.spec` to ensure `collect_all('torch')` continues capturing the new CUDA 12.6 runtime DLLs into root `_internal`.
4. **Offline Validation:** Execute `scratch/verify_st6.py` and Kokoro TTS speech tests with `HF_HUB_OFFLINE=1` to guarantee local execution.
5. **Full Suite Gate:** Require all 64 unit test suites and the 10/10 corpus gate to pass with exit code 0.

---

## 10. Unresolved Questions & Implementation Risks

1. **PyInstaller Freezing Layout:** PyTorch 2.10+ may reorganize internal DLL dependencies or CUDA 12.6 helper binaries. The PyInstaller freezing step (`pyinstaller sidecars/python/sidecar.spec`) must be verified to ensure no missing DLLs (e.g. `cublasLt64_12.dll` or `cudnn_adv64_9.dll`).
2. **BGE-M3 Safetensors Conversion:** While BGE-M3 currently functions with `pytorch_model.bin`, migrating BGE-M3 to `model.safetensors` would eliminate PyTorch unpickling for embeddings entirely. This can be evaluated as a secondary enhancement.
3. **VRAM Footprint on RTX 3050:** CUDA 12.6 runtime initialization memory must be measured to verify that peak VRAM remains under the 4GB hardware barrier during concurrent BGE-M3 warmup.

---

## 11. GO / NO-GO Recommendation

### **Recommendation: GO (With Staged Verification)**

- **Rationale:**
  - The vulnerability (CVE-2026-24747) is verified, active, and present in our current codebase via Kokoro's `.pth`/`.pt` loading paths.
  - An official, binary-compatible Windows x64 wheel (`torch==2.10.0+cu126`) exists for Python 3.10.
  - The host GPU and driver natively support CUDA 12.6.
  - The packaging payload remains well within our NSIS 2.0 GB budget.
  - Zero application code rewrites are required for Kokoro or sidecar routes.
  - A comprehensive cryptographic integrity verification architecture has been designed to complement the framework patch.

**Status:** Awaiting explicit authorization from Onii-sama before initiating Phase 2C installation.
