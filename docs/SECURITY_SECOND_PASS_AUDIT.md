# Omnitome — Second-Pass Security Audit & Forensic Verification

> **Document Status:** Authoritative Forensic Record  
> **Repository:** `ayumu-ml/Omnitome` (`Smart-Reader`)  
> **Date:** 2026-11-19  
> **Auditors:** Antigravity (Shiori) & Engineering Team  
> **Methodology:** Evidence-First Prime Directive (Zero Speculation, Zero False-Sense-of-Security Dismissal)

---

## Executive Summary

The initial triage dismissed several Dependabot and CodeQL alerts based on high-level architectural assumptions (namely that Omnitome is an offline/local desktop application). Following Hakari's rigorous security challenge, this **Second-Pass Audit** conducted code-level traces, package disassembly, opcode analysis, and network boundary testing.

### Key Corrections to Initial Audit:
1. **PyTorch (CVE-2026-24747):** The initial conclusion that `weights_only=True` eliminated risk was **incorrect**. CVE-2026-24747 is specifically a memory corruption vulnerability in PyTorch's `weights_only=True` unpickler itself.
2. **Sentence Transformers (CVE-2026-68770):** The vulnerability in `sentence_transformers/util/misc.py` bypasses `trust_remote_code=False` whenever the path exists locally on disk. Because Omnitome stores `bge-m3` locally under `%LOCALAPPDATA%` without file integrity verification, this is a **confirmed unpatched library flaw**.
3. **SSRF (`webFetcher.js`):** While literal private IPs are blocked, `webFetcher.js` does **not** perform pre-flight DNS resolution. Any domain resolving to `127.0.0.1` (e.g., DNS rebinding or `*.nip.io`) bypasses the hostname filter.
4. **ReDoS:** The table separator regex in [markdownParser.js](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/backend/services/ingestion/parsers/markdownParser.js#L129) is a **confirmed exponential backtracking defect** on malformed markdown books.

---

## 1. Deep Dive: PyTorch (CVE-2026-24747)

### 1.1 Environment & Version Verification
- **Installed Version:** `2.6.0+cu124` (verified in `sidecars/python/.venv/Scripts/python.exe`)
- **Manifest Version:** `torch==2.6.0+cu124` in [sidecars/python/requirements.txt](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/sidecars/python/requirements.txt)
- **Patched Upstream Version:** `torch >= 2.10.0`

### 1.2 Call Path Trace
A repository-wide AST audit found `torch.load` invoked at:
1. [sidecars/python/tts/synthesize.py:213](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/sidecars/python/tts/synthesize.py#L213):
   ```python
   pack = torch.load(v_path, weights_only=True)
   ```
2. `kokoro/model.py:68` (Inside Kokoro dependency):
   ```python
   torch.load(model, map_location='cpu', weights_only=True)
   ```
3. `kokoro/pipeline.py:147` (Inside Kokoro dependency):
   ```python
   torch.load(f, weights_only=True)
   ```

### 1.3 Forensic Flaw Mechanism in `weights_only=True`
CVE-2026-24747 (CVSS 8.8) affects `torch < 2.10.0`. The `weights_only=True` unpickler implementation contains two critical bugs:
- **Heap memory corruption:** `SETITEM` and `SETITEMS` pickle opcodes can be applied to non-dictionary objects, violating internal invariant checks.
- **Storage size mismatch:** Element counts declared in headers are not strictly validated against payload byte counts, triggering out-of-bounds writes.

Therefore, **invoking `torch.load(..., weights_only=True)` does NOT mitigate CVE-2026-24747.**

### 1.4 Attack Surface & Provencance
- **Checkpoint Sources:** Kokoro model checkpoint `kokoro-v1_0.pth` (~327 MB) and voice weights `voices/*.pt` (~523 KB each).
- **Integrity Boundary:** Currently, `synthesize.py` checks `os.path.isfile(v_path)`, but does **not** compute SHA-256 hashes or verify digital signatures before unpickling.
- **Threat Vector:** If an unprivileged local process or malware replaces or tampers with `af_heart.pt` in `%LOCALAPPDATA%\Omnitome\models\kokoro\voices\`, executing TTS will trigger memory corruption.
- **Upgrade Feasibility:** PyTorch 2.10.0+ is published on PyPI. Upgrading requires verifying CUDA 12.4 binary packaging against the **NSIS 2GB installer limit** (`resources/sidecar.7z`).

---

## 2. Deep Dive: Sentence Transformers (CVE-2026-68770)

### 2.1 Environment & Version Verification
- **Installed Version:** `3.0.1` (verified in `sidecars/python/.venv/Scripts/python.exe`)
- **Manifest Version:** `sentence-transformers==3.0.1` in [sidecars/python/requirements.txt](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/sidecars/python/requirements.txt)
- **Patched Upstream Version:** `sentence-transformers >= 5.6.0`

### 2.2 Call Path Trace
Model loading occurs at:
- [sidecars/python/embed/routes.py:73](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/sidecars/python/embed/routes.py#L73):
  ```python
  model = SentenceTransformer(model_id_or_path, model_kwargs=model_kwargs, device=device, local_files_only=True)
  ```
- [sidecars/python/nlp/routes.py:53](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/sidecars/python/nlp/routes.py#L53):
  ```python
  _reranker = CrossEncoder(model_id_or_path, max_length=512, device=device, automodel_args=model_kwargs)
  ```

### 2.3 Forensic Flaw Mechanism
In `sentence_transformers < 5.6.0`, `import_module_class` in `sentence_transformers/util/misc.py` evaluates:
```python
if trust_remote_code or os.path.exists(model_name_or_path):
    # Dynamically loads custom module defined in modules.json!
```
Because `os.path.exists(model_name_or_path)` is true for any local directory, the trust gate is **completely bypassed** on local directories.

### 2.4 Disk Analysis of Model Directory
Inspection of `C:\Users\Godsa\AppData\Local\Omnitome\models\sentence_transformers\models--BAAI--bge-m3`:
- Contains [modules.json](file:///C:/Users/Godsa/AppData/Local/Omnitome/models/sentence_transformers/models--BAAI--bge-m3/modules.json).
- If any malicious local software modifies `modules.json` to point to a custom `modeling_*.py`, Sentence Transformers executes that Python script upon application startup.
- **Classification:** **Critical Severity Unpatched Dependency**.
- **Remediation:** Upgrade `sentence-transformers` to `>= 5.6.0` (which eliminates the `os.path.exists` bypass) and implement SHA-256 manifest verification before loading local models.

---

## 3. Comprehensive SSRF Audit Matrix

User-controlled URL fetches originate in:
1. [backend/services/web/webFetcher.js](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/backend/services/web/webFetcher.js#L141) (Web Article Import)
2. [backend/routes/settingsRoutes.js](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/backend/routes/settingsRoutes.js#L122) (LLM Endpoint Verification)
3. [backend/services/ai/sidecarBase.js](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/backend/services/ai/sidecarBase.js#L199) (Internal Sidecar Dispatch)

### 3.1 Network Target Test Matrix (`webFetcher.js`)

| Target Scenario | Example Input | Current Status | Forensic Reason & Code Location |
| :--- | :--- | :--- | :--- |
| **IPv4 Loopback** | `http://127.0.0.1:8765/` | **BLOCKED** | Matched by `isRestrictedIpv4` (`b0 === 127`). |
| **Localhost Alias** | `http://localhost:3000/` | **BLOCKED** | Matched by `isRestrictedHost` (`hostname === 'localhost'`). |
| **IPv6 Loopback** | `http://[::1]:8080/` | **BLOCKED** | Matched by `isRestrictedHost` (`hostname === '::1'`). |
| **Private Class A** | `http://10.0.0.1/` | **BLOCKED** | Matched by `isRestrictedIpv4` (`b0 === 10`). |
| **Private Class B** | `http://172.16.0.1/` | **BLOCKED** | Matched by `isRestrictedIpv4` (`b0 === 172 && b1 >= 16 && b1 <= 31`). |
| **Private Class C** | `http://192.168.1.1/` | **BLOCKED** | Matched by `isRestrictedIpv4` (`b0 === 192 && b1 === 168`). |
| **Link-Local / Metadata** | `http://169.254.169.254/` | **BLOCKED** | Matched by `isRestrictedIpv4` (`b0 === 169 && b1 === 254`). |
| **CGNAT Block** | `http://100.64.0.1/` | **BLOCKED** | Matched by `isRestrictedIpv4` (`b0 === 100 && b1 >= 64`). |
| **IPv6 ULA / Link-Local** | `http://[fe80::1]/` | **BLOCKED** | Matched by `isRestrictedHost` (`/^fe[89ab][0-9a-f]:/i`). |
| **IPv4-Mapped IPv6** | `http://[::ffff:127.0.0.1]/` | **BLOCKED** | Handled in `isRestrictedHost` lines 105–120. |
| **Non-HTTP Protocol** | `file:///C:/Windows/` | **BLOCKED** | Guarded by `parsed.protocol !== 'http:' && parsed.protocol !== 'https:'`. |
| **Redirect to Internal** | `http://external.com/r` -> `http://127.0.0.1` | **BLOCKED** | Manual redirect loop in `webFetcher.js:230-242` inspects each intermediate hop. |
| **DNS Rebinding / nip.io** | `http://127.0.0.1.nip.io/` | **UNRESOLVED (VULNERABLE)** | Hostname `127.0.0.1.nip.io` is neither literal IP nor `.local`/`.internal`. `fetch()` resolves to `127.0.0.1` and executes connection. |
| **Cloud Metadata Rebinding** | `http://169.254.169.254.nip.io/` | **UNRESOLVED (VULNERABLE)** | Bypasses literal check; reaches AWS/GCP metadata if run on cloud developer machine. |

### 3.2 Settings Route Audit (`settingsRoutes.js`)
In `/api/settings/llm/test`:
- Accepts arbitrary `baseUrl`.
- Does **not** invoke `isRestrictedHost`.
- Intended for local Ollama (`http://localhost:11434`), but allows arbitrary outbound network probing.
- **Remediation:** Disallow link-local metadata addresses (`169.254.169.254`) while permitting verified localhost ports.

---

## 4. Deep Dive: Path Traversal

### 4.1 `backend/routes/ttsRoutes.js`
- **Route:** `GET /api/tts/audio/:hash?voice=af_heart`
- **Validation Chain:**
  1. `HASH_REGEX = /^[a-f0-9]{64}$/`: Validates that `hash` is strictly 64 hex characters.
  2. `VOICE_WHITELIST = ['af_heart', 'am_adam', 'bf_alice', 'bm_george']`: Restricts `voice` to exact match.
  3. `prefix = hash.slice(0, 2)`: Guaranteed 2 hex characters.
  4. `filePath = path.join(TTS_CACHE_DIR, normVoice, prefix, `${hash}.wav`)`.
- **Verdict:** **Confirmed False Positive for Direct Traversal**. Path components cannot contain `..` or separators.
- **Residual Risk (Defense-in-Depth):** Lack of `path.resolve(filePath).startsWith(path.resolve(TTS_CACHE_DIR))` and symlink resolution via `fs.realpathSync`.

### 4.2 `backend/services/ingestion/importQueueService.js`
- **Code:** [importQueueService.js:201-202](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/backend/services/ingestion/importQueueService.js#L201-L202):
  ```javascript
  } else if (file.path && fs.existsSync(file.path)) {
    await fs.promises.copyFile(file.path, stagingPath);
  }
  ```
- **Vulnerability:** `file.path` is copied directly into `config.STAGING_DIR` without validating that `file.path` resides inside a designated temporary upload folder.
- **Current Runtime Status:** HTTP routes currently configure Multer with `memoryStorage()` (supplying `file.buffer`). However, the method interface is exposed and unconstrained.
- **Classification:** **High-Priority Defense-in-Depth Fix**. Add path containment validation on `file.path`.

---

## 5. ReDoS Vulnerability Verification

### 5.1 Confirmed Defect: [markdownParser.js:129](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/backend/services/ingestion/parsers/markdownParser.js#L129)
- **Regex:** `/^\|?\s*:?-+:?\s*(\|?\s*:?-+:?\s*)+\|?$/`
- **Flaw:** Nested quantifiers with optional pipes and whitespace (`\s*` and `\|?` inside `()+`) cause catastrophic polynomial/exponential backtracking when matching strings like `|----------------------------...` lacking a terminating pipe.
- **Impact:** Freezes Node.js single-threaded event loop during book ingestion.
- **Remediation:** Replace with deterministic linear scanner:
  ```javascript
  function isTableSeparator(line) {
    const trimmed = line.trim();
    if (!trimmed.includes('-')) return false;
    const parts = trimmed.split('|').map(p => p.trim()).filter(Boolean);
    return parts.length > 0 && parts.every(p => /^:?-+:?$/.test(p));
  }
  ```

---

## 6. Actionable Remediation Roadmap

| Priority | Target Subsystem | Remediation Task | Blast Radius |
| :--- | :--- | :--- | :--- |
| **P0** | `backend/services/web/webFetcher.js` | Add pre-flight asynchronous DNS resolution (`dns.promises.lookup`) to validate resolved IPs against `isRestrictedIpv4`. | Low; web import only |
| **P0** | `backend/services/ingestion/parsers/markdownParser.js` & `aiNormalizer.js` | Replace backtracking regex with linear table delimiter scanner. | Low; parser only |
| **P1** | `backend/services/ingestion/importQueueService.js` | Enforce directory containment check on `file.path` before `fs.promises.copyFile`. | Minimal |
| **P1** | `backend/routes/ttsRoutes.js` | Enforce `path.resolve(filePath).startsWith(path.resolve(TTS_CACHE_DIR))`. | None (redundant defense) |
| **P1** | `sidecars/python/requirements.txt` | Upgrade `sentence-transformers` from `3.0.1` to `>= 5.6.0` and `python-multipart` to `>= 0.0.30`. | Validate BGE-M3 loading |
| **P2** | `sidecars/python/` routes | Sanitize exception handling in FastAPI routes (log traceback to stderr, return structured message). | Low; sidecar API |
| **P2** | `sidecars/python/requirements.txt` | Upgrade `torch` to `>= 2.10.0` with full CUDA 12.4 and PyInstaller NSIS bundle verification. | Sidecar packaging gate |

---

## Associated Inventories & Deliverables
- Complete 74-Alert Itemized Inventory: [DEPENDABOT_ALERT_INVENTORY.md](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/docs/DEPENDABOT_ALERT_INVENTORY.md)
- Complete 90-Finding CodeQL Register: [CODEQL_TRIAGE.md](file:///c:/My%20Personal%20Folders/Projects%20to%20work%20on/Antigravity%20Projects/Smart-Reader/docs/CODEQL_TRIAGE.md)
