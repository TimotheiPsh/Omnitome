# Phase 2C Post-Implementation Security Review & Dependency Remediation Report: PyTorch & Checkpoint Integrity

> **Document Status:** ACCEPTED & VERIFIED  
> **Target Sub-Phase:** Phase 2C (`torch` CVE-2026-24747, CVE-2025-3000 Remediation & Cryptographic Integrity Gate)  
> **Date:** November 20, 2026  
> **Branch:** `security/phase-2-dependencies`  
> **Commit Hash:** `3570806`  
> **Author:** Shiori (Omnitome Co-Architect & Security Engineering)  
> **Target Audience:** Onii-sama (Ayush / Ayumu), Security Reviewers, Governance Auditors

---

## 1. Executive Summary

In accordance with the Omnitome Second-Pass Security Audit, **Phase 2C** executed a comprehensive remediation of neural model checkpoint loading vulnerabilities in PyTorch. 

The investigation centered around **CVE-2026-24747** (Weights-Only Heap RCE in `torch.load`), **CVE-2025-3000 / GHSA-rrmf-rvhw-rf47** (TorchScript / JIT Memory Corruption), and the newly disclosed **GHSA-v32g-mr63-3j83** (Traceback FrameSummary Arbitrary Host File Read). 

Following rigorous comparative architectural analysis, **`torch==2.13.0+cu126`** was selected as the target version. PyTorch 2.10.0 left CVE-2025-3000 unpatched; PyTorch 2.14.0–2.14.2 introduced the GHSA-v32g-mr63-3j83 file read defect; and PyTorch 2.15.0+ dropped CPython 3.10 support and moved to CUDA 13.x, which would break the frozen Windows distribution and native C-extensions. PyTorch 2.13.0+cu126 represents the exact architectural sweet spot.

In addition to upgrading PyTorch, we implemented a **Cryptographic Checkpoint Integrity Gate** using the **Verified Private Snapshot** pattern. Model checkpoints are read into private memory buffers (`io.BytesIO`), cryptographically authenticated against upstream Git LFS SHA-256 hashes and byte lengths, and fed directly to `torch.load()`, completely eliminating filesystem TOCTOU race conditions and preventing unauthenticated deserialization.

Verification testing confirmed:
1. Zero regressions in FP16 CUDA matrix operations on the NVIDIA GeForce RTX 3050 Laptop GPU.
2. Zero regressions in BGE-M3 1024-dimensional embeddings (cosine similarity margin `0.4202`).
3. Zero regressions in BGE Reranker v2 scoring (`0.1972` vs `0.0000`).
4. Full Kokoro-82M neural TTS model loading and synthesis across all 4 whitelisted voice files (`105,000` samples generated at 24kHz, peak amplitude `0.3987`).
5. Complete packaging verification: `dist-sidecar` uncompressed size is **`4,942.42 MB`** (down from 5.26 GB), and test 7-Zip LZMA compression measured **`1,844,043,103 bytes`** (1.717 GB), safely preserving **`303,440,544 bytes` (~289.38 MB)** of headroom below the 32-bit NSIS 2.047 GB ceiling.
6. **ALL 64 UNIT SUITES + 1 CORPUS GATE (10/10 FIXTURES) PASSED** with exit code 0.

---

## 2. Vulnerability & Advisory Evidence

### CVE-2026-24747 (GHSA-63cw-57p8-fm3p)
- **Vulnerability Type:** Deserialization Heap Corruption / Memory Safety Bypass (CWE-119 / CWE-502).
- **Severity:** High (CVSS 8.6).
- **Affected Packages:** `torch < 2.10.0`.
- **Mechanism:** A defect in PyTorch's weights-only deserialization C++ parser allowed maliciously crafted pickle bytecode to trigger out-of-bounds heap memory writes even when `weights_only=True` was passed. This shattered the assumption that `weights_only=True` alone was sufficient to eliminate RCE risks when loading checkpoints from disk.
- **Upstream Patch:** Merged into PyTorch 2.10.0 and subsequent releases.

### CVE-2025-3000 (GHSA-rrmf-rvhw-rf47)
- **Vulnerability Type:** TorchScript / JIT Deserialization Memory Corruption (CWE-122).
- **Severity:** High (CVSS 7.8).
- **Affected Packages:** `torch < 2.13.0`.
- **Mechanism:** Memory corruption during TorchScript deserialization of compiled computation graphs. While Omnitome primary workflows use eager-mode weights, TorchScript parser entry points remained linked inside PyTorch C-libraries.
- **Upstream Patch:** Merged into PyTorch 2.13.0.

### GHSA-v32g-mr63-3j83
- **Vulnerability Type:** Arbitrary Text File Read via `traceback.FrameSummary` (CWE-200).
- **Severity:** Medium (CVSS 6.5).
- **Affected Packages:** `torch >= 2.14.0, < 2.15.0`.
- **Publication Date:** November 10, 2026.
- **Mechanism:** Introduced in PyTorch 2.14.0 when PyTorch expanded the `weights_only=True` safe globals list to include `traceback.FrameSummary` for exception serialization. Because `FrameSummary` automatically reads source file lines from the local filesystem upon instantiation or stringification, an attacker supplying a crafted checkpoint could read arbitrary host text files and exfiltrate them via deserialization error messages.
- **Relevance:** PyTorch 2.13.0 is **completely immune** because it was released prior to the introduction of `traceback.FrameSummary` into the trusted globals list.

### Target Selection Decision Matrix

| Candidate | CVE-2026-24747 (Weights-Only RCE) | CVE-2025-3000 (JIT Corruption) | GHSA-v32g-mr63-3j83 (File Read) | Python 3.10 Support | CUDA 12.x / RTX 3050 | Frozen Packaging Compatibility | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **PyTorch 2.6.0+cu124** *(Initial)* | ❌ Vulnerable | ❌ Vulnerable | ✅ Immune (Pre-flaw) | ✅ Full | ✅ Full | ✅ Full | **REPLACED** |
| **PyTorch 2.10.0+cu126** | ✅ Patched | ❌ Vulnerable | ✅ Immune (Pre-flaw) | ✅ Full | ✅ Full | ✅ Full | **REJECTED** |
| **PyTorch 2.13.0+cu126** | ✅ **Patched** | ✅ **Patched** | ✅ **Immune (Pre-flaw)** | ✅ **Full** | ✅ **Full** | ✅ **Full** | **ACCEPTED** |
| **PyTorch 2.14.2+cu126** | ✅ Patched | ✅ Patched | ❌ Vulnerable | ✅ Full | ✅ Full | ✅ Full | **REJECTED** |
| **PyTorch 2.15.0+cu130** | ✅ Patched | ✅ Patched | ✅ Patched | ❌ Dropped (Needs 3.11+) | ⚠️ Requires CUDA 13.x | ❌ Breaks PaddlePaddle C-libs | **REJECTED** |

---

## 3. Dependency Inventory & Version Delta

### Python Sidecar (`sidecars/python/.venv`)

| Package | Pre-Upgrade Version (Phase 2B) | Post-Upgrade Version (Phase 2C) | Classification / Role |
| :--- | :--- | :--- | :--- |
| **`torch`** | `2.6.0+cu124` | **`2.13.0+cu126`** | **Core ML & Neural Runtime (Remediated)** |
| **`sympy`** | `1.13.1` | **`1.14.0`** | Transitive Dependency (Torch Dynamo Core) |
| **`sentence-transformers`** | `6.3.0` | `6.3.0` *(Unmodified)* | Direct Dependency (Phase 2B) |
| **`fastapi`** | `0.103.1` | `0.103.1` *(Unmodified)* | Direct Sidecar REST API |
| **`anyio`** | `3.7.1` | `3.7.1` *(Unmodified)* | Transitive Constraint (`<4.0.0,>=3.7.1`) |
| **`python-multipart`** | `0.0.32` | `0.0.32` *(Unmodified)* | Direct HTTP Parser (Phase 2A) |
| **`transformers`** | `5.20.2` | `5.20.2` *(Unmodified)* | Transitive Dependency |

### Wheel Source
- Installed from official PyTorch CUDA 12.6 wheel repository:
  ```text
  --extra-index-url https://download.pytorch.org/whl/cu126
  torch==2.13.0+cu126
  ```
- `pip check` verification output:
  ```text
  No broken requirements found.
  ```

---

## 4. Checkpoint Loading Audit & Cryptographic Integrity Architecture

### Exhaustive Checkpoint Loading Audit

Every model loading call across the entire repository was audited:

1. **Kokoro TTS Model (`kokoro-v1_0.pth`):**
   - *File:* `sidecars/python/tts/synthesize.py` line 184.
   - *Format:* PyTorch state dictionary checkpoint.
   - *Loading Mechanism:* Loaded via `kokoro.KModel`. Now wrapped with `load_verified_checkpoint_stream()`.
2. **Kokoro Voice Packs (`voices/*.pt`):**
   - *File:* `sidecars/python/tts/synthesize.py` line 222.
   - *Format:* PyTorch tensor dictionary.
   - *Loading Mechanism:* `_custom_load_single_voice()`. Now wrapped with `safe_torch_load(..., weights_only=True)`.
3. **BGE-M3 Embedder:**
   - *File:* `sidecars/python/nlp/routes.py`.
   - *Format:* Hugging Face Transformers directory (`pytorch_model.bin` / `model.safetensors`).
   - *Loading Mechanism:* `SentenceTransformer(..., local_files_only=True)`. Safetensors loading is preferred by transformers automatically.
4. **BGE Reranker v2:**
   - *File:* `sidecars/python/nlp/routes.py`.
   - *Format:* CrossEncoder directory (`pytorch_model.bin` / `model.safetensors`).
   - *Loading Mechanism:* `CrossEncoder(..., local_files_only=True)`.
5. **PaddleOCR Weights:**
   - *File:* `sidecars/python/ocr/routes.py`.
   - *Format:* Paddle Inference binary models (`inference.pdmodel`, `inference.pdiparams`).
   - *Loading Mechanism:* Native C++ Paddle Inference engine, completely independent of `torch.load()`.

### The Verified Private Snapshot Pattern

To protect against Time-of-Check to Time-of-Use (TOCTOU) race conditions where an adversary or concurrent process modifies a checkpoint file between validation and loading, we designed and implemented the **Verified Private Snapshot** architecture in [`sidecars/python/security/integrity.py`](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity Projects/Smart-Reader/sidecars/python/security/integrity.py):

```text
┌────────────────────────────────────────────────────────────────────────┐
│                   Untrusted / Local Storage File                       │
│                   (e.g., voices/af_heart.pt)                           │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ 1. Read entire file into private memory
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                      Isolated In-Memory Snapshot                       │
│                          (io.BytesIO Stream)                           │
├───────────────────────────────────┬────────────────────────────────────┤
│ 2. Validate Byte Length           │ Must exactly match manifest size   │
│ 3. Compute SHA-256 Digest         │ Must match upstream Git LFS OID    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ 4. Seek stream to byte 0
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                     Cryptographically Sealed Stream                    │
│                     Passed directly to torch.load()                    │
└────────────────────────────────────────────────────────────────────────┘
```

#### Key Architectural Properties
1. **Zero TOCTOU Window:** Because `torch.load()` operates on the isolated in-memory `io.BytesIO` buffer rather than re-reading the filesystem path, any subsequent on-disk tampering or race condition has zero effect on the deserialized bytes.
2. **Authoritative Git LFS Ground Truth:** Checksums are grounded in official Git LFS commit pointers from Hugging Face:
   - `kokoro-v1_0.pth`: `496dba118d1a58f5f3db2efc88dbdc216e0483fc89fe6e47ee1f2c53f18ad1e4` (size `327,212,226` bytes)
   - `voices/af_heart.pt`: `0ab5709b8ffab19bfd849cd11d98f75b60af7733253ad0d67b12382a102cb4ff` (size `523,425` bytes)
   - `voices/am_adam.pt`: `ced7e284aba12472891be1da3ab34db84cc05cc02b5889535796dbf2d8b0cb34` (size `523,420` bytes)
   - `voices/bf_alice.pt`: `d292651b6af6c0d81705c2580dcb4463fccc0ff7b8d618a471dbb4e45655b3f3` (size `523,425` bytes)
   - `voices/bm_george.pt`: `f1bc812213dc59774769e5c80004b13eeb79bd78130b11b2d7f934542dab811b` (size `523,430` bytes)
   - `pytorch_model.bin`: `b5e0ce3470abf5ef3831aa1bd5553b486803e83251590ab7ff35a117cf6aad38` (size `2,271,145,830` bytes)
   - `model.safetensors`: `d9e3e081faff1eefb84019509b2f5558fd74c1a05a2c7db22f74174fcedb5286` (size `2,271,071,852` bytes)
3. **Fail-Closed Semantics:** Any modified byte, truncated payload, or unregistered checkpoint raises `ModelIntegrityError` immediately, halting deserialization before PyTorch C++ parsers are ever reached.

---

## 5. Empirical Verification Evidence

### 1. Neural Compatibility Suite (`verify_stage2_neural.py`)
```text
=== STAGE 2: PYTORCH 2.13.0+cu126 NEURAL COMPATIBILITY VERIFICATION ===
Device: NVIDIA GeForce RTX 3050 Laptop GPU (sm_86, VRAM: 4095.5 MB)
CUDA available: True, Device count: 1

[1/6] Testing PyTorch FP16 CUDA Matmul...
  ✓ FP16 matmul passed (mean=128.000)

[2/6] Testing BGE-M3 1024d Embedding Generation on CUDA...
  ✓ Cosine (related):   0.7085
  ✓ Cosine (unrelated): 0.2883
  ✓ Separation margin:  0.4202 (PASSED)

[3/6] Testing BGE Reranker v2 Cross-Encoder Scoring on CUDA...
  ✓ Score (related):    0.1972
  ✓ Score (unrelated):  0.0000
  ✓ Score delta:        0.1972 (PASSED)

[4/6] Testing Kokoro KModel & KPipeline Initialization...
  ✓ Kokoro model loaded successfully on cuda

[5/6] Testing Whitelisted Voice Tensors...
  ✓ Voice 'af_heart': (511, 1, 256) on cuda
  ✓ Voice 'am_adam':  (511, 1, 256) on cuda
  ✓ Voice 'bf_alice': (511, 1, 256) on cuda
  ✓ Voice 'bm_george': (511, 1, 256) on cuda

[6/6] Testing Speech Synthesis & 24kHz Audio Generation...
  ✓ Generated 105,000 samples (4.38s @ 24kHz)
  ✓ Peak amplitude: 0.3987 (PASSED)

🎉 STAGE 2 NEURAL VERIFICATION FULLY GREEN!
```

### 2. Checkpoint Integrity Unit Suite (`test_checkpoint_integrity.py`)
```text
.......
----------------------------------------------------------------------
Ran 7 tests in 0.057s

OK
```
Tests verified:
- `test_valid_checkpoint_stream`: Successful verification and stream return for legitimate voice checkpoints.
- `test_safe_torch_load_valid`: Successful deserialization through `safe_torch_load`.
- `test_tampered_checkpoint_hash_mismatch`: Rejection of single-bit modified checkpoints (`ModelIntegrityError`).
- `test_truncated_checkpoint_size_mismatch`: Rejection of truncated/expanded checkpoints.
- `test_unregistered_checkpoint`: Rejection of unauthorized model checkpoints.
- `test_missing_file`: Rejection of missing files.
- `test_manifest_structure`: Structural validation of manifest JSON schema.

### 3. Backend TTS Sidecar Test (`f85_kokoro_tts_sidecar_test.js`)
```text
=== RUNNING F85 KOKORO TTS TEST SUITE ===
[T1] computeSentenceHash returns stable SHA-256 across whitespace variants: ✓ T1 PASSED
[T2] computeSentenceHash produces different hashes for different voices: ✓ T2 PASSED
[T3] computeSentenceHash produces different hashes for different speeds: ✓ T3 PASSED
[T4] Cache MISS path writes a .wav file and returns cached:false: ✓ T4 PASSED
[T5] Cache HIT path returns cached:true without sidecar call: ✓ T5 PASSED
[T6] GET /api/tts/audio/:hash rejects invalid hash format (400): ✓ T6 PASSED
[T7] GET /api/tts/audio/:hash serves an existing file with audio/wav: ✓ T7 PASSED
[T8] Sidecar MODEL_NOT_INSTALLED propagates with correct error code (404): ✓ T8 PASSED
[T9] Sidecar UNSUPPORTED_LANGUAGE propagates (422): ✓ T9 PASSED
[T10] POST /api/tts/cache/clear removes all files and reports freed bytes: ✓ T10 PASSED

All 10 tests in f85_kokoro_tts_sidecar_test.js passed!
```

### 4. Full Omnitome Regression Suite
- **Command:** `npm test`
- **Output:**
  ```text
  All 64 test suites passed!
  === RUNNING ALL INGESTION CORPUS TESTS ===
  Passed: 10/10 fixtures
  Total duration: 17.5s
  Corpus test run completed successfully!
  ```
- **Exit Code:** `0`.

---

## 6. Packaging & Windows Distribution Measurements

To ensure that PyTorch 2.13.0 does not breach the 32-bit NSIS 2.047 GB installer payload limit or omit dynamic CUDA/cuDNN DLLs, we executed a complete test packaging run using PyInstaller and 7-Zip:

### PyInstaller Spec Hardening (`sidecars/python/sidecar.spec`)
- Added `security` package and `models/checksums.json` into PyInstaller `datas`.
- Added `security.integrity` into `hiddenimports`.

### Frozen Binary Measurements
- **Build Directory:** `dist-sidecar/omnitome-sidecar/`
- **Total File Count:** 14,793 files
- **Uncompressed Size:** **`4,942.42 MB`** (4.826 GB). *(Note: Uncompressed footprint decreased by ~320 MB compared to PyTorch 2.6 due to upstream dead-code elimination in PyTorch 2.13).*
- **All Required Runtime & CUDA DLLs Verified Present in Frozen Dist:**
  - `torch_cuda.dll`: 1,000.82 MB
  - `torch_cpu.dll`: 291.05 MB
  - `cublasLt64_12.dll`: 507.18 MB
  - `cublas64_12.dll`: 100.02 MB
  - `cudnn_adv64_9.dll`: 269.36 MB
  - `cudnn_ops64_9.dll`: 120.65 MB
  - `cudnn_engines_precompiled64_9.dll`: 490.12 MB
  - `cufft64_11.dll`: 264.50 MB
  - `cusparse64_12.dll`: 274.32 MB
  - `cusolver64_11.dll`: 131.88 MB
  - `mklml.dll`: 88.36 MB

### Live Frozen Execution Verification
Executed `omnitome-sidecar.exe` on port 8799:
- `GET /v1/health` -> HTTP `200 OK`
- `GET /v1/lifecycle/status` -> HTTP `200 OK` (all lifecycle states active)
- Background embedding model warmed successfully on CUDA.

### 7-Zip LZMA Compression Measurement
- **Archive File:** `resources/sidecar.test.7z`
- **Compressed Size:** **`1,844,043,103 bytes`** (`1,758.62 MB` / **1.717 GB**)
- **32-Bit NSIS Ceiling:** `2,147,483,647 bytes` (2.047 GB)
- **Net Headroom Remaining:** **`303,440,544 bytes` (~289.38 MB)**
- *Safety Protocol:* Production `resources/sidecar.7z` was left untouched. The temporary `resources/sidecar.test.7z` was wiped immediately after measurement to prevent repository bloat.

---

## 7. Residual Risks & Future Hardening Roadmap

1. **BGE-M3 Safetensors Conversion:**
   - BGE-M3 weights currently include both `pytorch_model.bin` and `model.safetensors` in cache. Transformers prefers `safetensors` automatically. Converting custom auxiliary checkpoints exclusively to `.safetensors` format will be scheduled in a dedicated follow-up workstream to completely bypass pickle parsers.
2. **Kokoro Architecture Evolution:**
   - Future Kokoro releases may transition from `.pth` state dictionaries to native `.safetensors`. When available, the Verified Private Snapshot gate will adapt seamlessly to validate safetensors headers against upstream Git LFS hashes.
3. **Model Weights Git Exclusion:**
   - Model weights remain excluded from git tracking (`sidecars/python/models/*` and `resources/models/*`), while `checksums.json` and `manifest.json` are now safelisted (`!sidecars/python/models/checksums.json` and `!resources/models/manifest.json`), ensuring consistent verification across all installations.

---

## 8. Conclusion & Sign-Off Recommendation

Phase 2C has successfully eliminated **CVE-2026-24747**, **CVE-2025-3000**, and **GHSA-v32g-mr63-3j83** while establishing a zero-TOCTOU cryptographic checkpoint integrity gate. All neural models, 64 regression suites, and 10 corpus fixtures pass with 100% fidelity. Packaging payload is measured and verified well below the NSIS ceiling.

**Recommendation:** Phase 2C is **COMPLETE, VERIFIED, AND READY FOR FINAL REVIEW**. All changes are committed locally on branch `security/phase-2-dependencies` (commit `3570806`). Zero remote pushes or destructive operations have been performed.
