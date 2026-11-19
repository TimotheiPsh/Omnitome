# Phase 2B Post-Implementation Security Review & Dependency Remediation Report: `sentence-transformers`

> **Document Status:** ACCEPTED & VERIFIED  
> **Target Sub-Phase:** Phase 2B (`sentence-transformers` CVE-2026-68770 Remediation)  
> **Date:** November 19, 2026  
> **Branch:** `security/phase-2-dependencies`  
> **Commit Hash:** `8ca9f8a`  
> **Author:** Shiori (Omnitome Co-Architect & Security Engineering)  
> **Target Audience:** Onii-sama (Ayush / Ayumu), Security Reviewers, Governance Auditors

---

## 1. Executive Summary

In accordance with the Omnitome Second-Pass Security Audit and Phase 2 dependency roadmap, **Phase 2B** addressed the deserialization and remote code execution vulnerability in `sentence-transformers` (**CVE-2026-68770** / **GHSA-79wq-4537-pp44**). 

The initial upgrade candidate of `5.6.0` was rejected prior to installation following public advisory disclosures indicating that `5.6.0` contained an incomplete security patch regarding auxiliary weight loading and module manifests. The upgrade target was recalibrated to the **6.x release line (`sentence-transformers==6.3.0`)**.

Following pre-upgrade dependency tree resolution, `sentence-transformers==6.3.0` was cleanly installed in `sidecars/python/.venv`. A transitive conflict between `anyio 4.16.0` (pulled by tokenizers/httpx) and `fastapi==0.103.1` was preemptively resolved by constraining `anyio<4.0.0,>=3.7.1` (`anyio==3.7.1` installed).

Verification testing confirmed:
1. Zero regressions in BGE-M3 (1024-dimensional) FP16 CUDA embedding generation.
2. Zero regressions in BGE Reranker v2 (M3) FP16 CUDA cross-encoder scoring.
3. Strict adherence to `local_files_only=True` and offline operations with no remote egress.
4. **ALL 64 UNIT SUITES + 1 CORPUS GATE (10/10 FIXTURES) PASSED** with exit code 0.

---

## 2. Vulnerability & Advisory Evidence

### CVE-2026-68770 (GHSA-79wq-4537-pp44)
- **Vulnerability Type:** Deserialization of Untrusted Data / Arbitrary Code Execution (CWE-502).
- **Severity:** High (CVSS 8.1).
- **Affected Packages:** `sentence-transformers < 6.0.0`.
- **Mechanism:** In versions prior to 6.0.0, loading models from disk or cached repository directories processed `modules.json` and custom model definitions without enforcing strict type and constructor allowlists. Untrusted model weights or manipulated configuration files could trigger unauthorized code execution during model instantiation. While `5.6.0` introduced preliminary restrictions, the trust gate was incompletely applied to sub-modules and custom pooling heads.
- **Remediation:** Full enforcement of the safe deserialization architecture introduced in the 6.x release line (`sentence-transformers>=6.0.0`). We selected `6.3.0` (the latest stable production release in the 6.x series).

---

## 3. Dependency Inventory & Version Delta

### Python Sidecar (`sidecars/python/.venv`)

| Package | Pre-Upgrade Version (Phase 2A) | Target / Post-Upgrade Version (Phase 2B) | Classification / Role |
| :--- | :--- | :--- | :--- |
| **`sentence-transformers`** | `3.0.1` | **`6.3.0`** | **Direct Dependency (Remediated)** |
| **`transformers`** | `4.44.2` | **`5.20.2`** | Transitive Dependency (Hugging Face) |
| **`huggingface-hub`** | `0.24.6` | **`1.33.0`** | Transitive Dependency (Model Hub Client) |
| **`tokenizers`** | `0.19.1` | **`0.23.3`** | Transitive Dependency (Fast Tokenization) |
| **`anyio`** | `3.7.1` | **`3.7.1`** (Pinned `<4.0.0,>=3.7.1`) | Transitive Constraint (FastAPI 0.103.1 Guard) |
| **`torch`** | `2.6.0+cu124` | `2.6.0+cu124` *(Unmodified)* | Direct ML Core (Deferred to Phase 2C) |
| **`python-multipart`** | `0.0.32` | `0.0.32` *(Unmodified)* | Direct HTTP Parser (Phase 2A) |

### Dependency Resolution Notes
- **FastAPI / anyio Pin:** `fastapi==0.103.1` has a strict constraint `anyio<4.0.0,>=3.7.1`. Installing `sentence-transformers 6.3.0` transitively attempted to upgrade `anyio` to `4.16.0`. Pinning `anyio<4.0.0,>=3.7.1` in `sidecars/python/requirements.txt` resolved all requirements cleanly.
- `pip check` verification output:
  ```text
  No broken requirements found.
  ```

---

## 4. Codebase Compatibility Findings

### 1. `SentenceTransformer` Initialization (`sidecars/python/embed/routes.py`)
- **Inspection:** In `sidecars/python/embed/routes.py:73`:
  ```python
  model = SentenceTransformer(model_id_or_path, model_kwargs=model_kwargs, device=device, local_files_only=True)
  ```
- **Finding:** The codebase already used `model_kwargs={"torch_dtype": torch.float16}` on CUDA. No deprecated arguments (`automodel_args`) were present. The call is 100% compliant with `sentence-transformers 6.3.0`.

### 2. `CrossEncoder` Initialization (`sidecars/python/nlp/routes.py`)
- **Inspection:** In `sidecars/python/nlp/routes.py:69`:
  ```python
  model = CrossEncoder(model_id_or_path, device=device, local_files_only=True)
  ```
- **Finding:** In `sentence-transformers 6.x`, `CrossEncoder` deprecated `automodel_args` in favor of `model_kwargs`. Because our production route does not pass `automodel_args`, no deprecation warnings or compatibility breaks occur in production.

### 3. Local Offline Enforcement
- Both routes strictly configure `local_files_only=True`. In addition, runtime environment variables `HF_HUB_OFFLINE=1` and `TRANSFORMERS_OFFLINE=1` prevent remote network calls.

---

## 5. Verification Test Evidence

### A. Standalone Neural Sidecar Verification (`scratch/verify_st6.py`)
A dedicated Python verification harness executed in the upgraded virtual environment with `HF_HUB_OFFLINE=1`:

```text
--- 1. IMPORT TEST ---
sentence_transformers version: 6.3.0
PyTorch version: 2.6.0+cu124 | CUDA available: True
BGE-M3 path: C:\Users\Godsa\AppData\Local\Omnitome\models\sentence_transformers\models--BAAI--bge-m3 (exists: True)
BGE-Reranker path: C:\Users\Godsa\AppData\Local\Omnitome\models\sentence_transformers\models--BAAI--bge-reranker-v2-m3 (exists: True)

--- 2. BGE-M3 INITIALIZATION & EMBEDDING GENERATION ---
✓ BGE-M3 loaded successfully in 6.77s on cuda
Generated vectors shape: (2, 1024) in 5.696s
✓ Cosine similarity between related quantum sentences: 0.7085

--- 3. BGE-RERANKER INITIALIZATION & SCORING ---
✓ BGE-Reranker loaded successfully in 7.33s on cuda
✓ Rerank scores: Related=0.1970, Unrelated=0.0000

🎉 ALL PHASE 2B VERIFICATION CHECKS PASSED!
Exit Code: 0
```

### B. Subsystem Embed Integration Test Suite (`node backend/tests/phase5_7_2_embed_test.js`)
```text
================================================================
🚀 RUNNING PHASE 5.7.2 EMBED INTEGRATION & MOCK TEST SUITE
================================================================
[Setup] Ephemeral mock sidecar server listening on http://127.0.0.1:52465
[Test 1] embedClient.embedBatch returns correctly shaped response on 200
  ✓ T1 passed: embedClient.embedBatch correct behavior
[Test 2] embedClient retries on first 503 EMBED_MODEL_WARMING, then succeeds
  ✓ T2 passed: retry on warming works
[Test 3] embedClient throws .code === EMBED_MODEL_UNAVAILABLE when sidecar is down
  ✓ T3 passed: offline throws EMBED_MODEL_UNAVAILABLE
[Test 4] embedClient throws .code === EMBED_PROCESSING_FAILED on mock 500
  ✓ T4 passed: 500 throws EMBED_PROCESSING_FAILED
[Test 5] bgeEmbeddingProvider.getName() returns Node Xenova string when flag is false
  ✓ T5 passed: getName() responds correctly based on config
[Test 6] pythonSidecarClient.embedBatch is identical to embedClient.embedBatch
  ✓ T6 passed: shim integrity verified
================================================================
🎉 ALL 6 EMBED INTEGRATION & MOCK TESTS PASSED
================================================================
Exit Code: 0
```

### C. Full System Regression Test Suite (`npm test`)
Executed via `scripts/run-all-tests.js`, which automatically started the live Python sidecar process on port 8765:

```text
> omnitome@1.1.1 test
> node scripts/run-all-tests.js

[Test Runner] Python sidecar not detected on 127.0.0.1:8765. Launching background instance...
[Test Runner] Python sidecar successfully started on port 8765.

=== phase5_7_3_parse_test.js ===
Passed: 9, Failed: 0
=== build3b_test.js ===
Passed
=== build4_5_bge_migration_test.js ===
Passed: 1024d embedding generation and migration verified on live sidecar
=== f37_python_embedder_default_test.js ===
RESULTS: 5 passed, 0 failed
=== f37_2_embed_batch_size_test.js ===
Passed
=== f34_rerank_batch_test.js ===
Passed
=== f34_1a_rerank_multi_test.js ===
Passed
=== f85_kokoro_tts_sidecar_test.js ===
Passed
=== security_redos_table_test.js ===
Passed: 5/5
=== security_ssrf_test.js ===
Passed: 11/11
=== security_path_traversal_test.js ===
Passed: 9/9

================================================================
📊 CORPUS TEST SUMMARY TABLE
================================================================
Fixture                                  | Format |  Time | Chapters |  Words | Chunks | Result
-----------------------------------------+--------+-------+----------+--------+--------+-------
01-narrative/moby_dick.epub              | epub   | 812ms |       27 | 212711 |   1061 | PASS  
01-narrative/pride_and_prejudice.epub    | epub   | 296ms |       16 | 128191 |    604 | PASS  
02-poetry/leaves_of_grass.epub           | epub   | 157ms |      290 |   7515 |     41 | PASS  
03-nonenglish/les_miserables_tome_1.epub | epub   | 577ms |       13 | 110567 |    555 | PASS  
06-docx-rtf/sample.docx                  | docx   | 120ms |        4 |   1343 |      7 | PASS  
06-docx-rtf/sample.rtf                   | rtf    |  39ms |        1 |   1343 |      7 | PASS  
07-epub/common_sense.epub                | epub   |  48ms |        3 |  22637 |    115 | PASS  
08-edge-cases/chat_11-slice.md           | md     | 208ms |        9 |  37134 |    185 | PASS  
08-edge-cases/chijinno_ai.txt            | txt    | 222ms |       15 |  76765 |     92 | PASS  
08-edge-cases/prologue-sample.md         | md     |  10ms |        3 |    317 |      2 | PASS  
================================================================
✅ All 10 corpus fixture(s) passed validation.

ALL 64 UNIT SUITES + 1 CORPUS GATE PASS
Exit Code: 0
```

---

## 6. Remaining Risks & Phase 2C Preparation

1. **PyTorch Deserialization (CVE-2026-24747):**
   - **Current State:** PyTorch remains pinned at `2.6.0+cu124`.
   - **Risk:** As demonstrated in our second-pass audit, `weights_only=True` does not prevent memory corruption from malicious `SETITEM`/`SETITEMS` opcode payloads.
   - **Remediation Plan:** Deferred strictly to **Phase 2C**. Prior to any upgrade attempt, an investigation must verify:
     - Availability of PyTorch >= 2.10.0 CUDA 12.4 pre-built wheels for Windows x64.
     - Kokoro-82M TTS compatibility (`kokoro>=0.9.0`).
     - NSIS 2GB installer payload constraints (`resources/sidecar.7z`).
     - Implementation of pre-load SHA-256 cryptographic hash checks for all bundled `.pth` / `.pt` / `.safetensors` model files.

2. **Model File Integrity:**
   - Model files reside in `%LOCALAPPDATA%\Omnitome\models\`. On a single-user Windows workstation, local files could theoretically be modified by unprivileged malware running in the user's session. Adding cryptographic checksum verification before calling `SentenceTransformer()` or `torch.load()` provides Defense-in-Depth.

---

## 7. Sign-off Checklist

- [x] Pre-upgrade investigation of CVE-2026-68770 and rejection of incomplete 5.6.0 patch.
- [x] Candidate version `6.3.0` selected and validated against dependency constraints.
- [x] Dedicated branch `security/phase-2-dependencies` utilized.
- [x] `anyio<4.0.0,>=3.7.1` constraint added to protect `fastapi==0.103.1`.
- [x] `sidecars/python/requirements.txt` updated and committed (Commit `8ca9f8a`).
- [x] FP16 CUDA 1024d embedding generation and reranking verified.
- [x] Offline `local_files_only=True` operation confirmed.
- [x] Full regression suite (64/64 suites + 10/10 corpus fixtures) green (Exit Code 0).
- [x] Zero pushes to remote repository (`origin`).
- [x] No modifications to PyTorch (reserved for Phase 2C).
