# Dependabot Alert Inventory (Second-Pass Audit)

> **Repository:** `ayumu-ml/Omnitome`  
> **Audit Date:** 2026-11-19  
> **Total Alerts Triaged:** 74  
> **Status:** Completed forensic second-pass assessment without unevidenced dismissal.

---

## Executive Summary

This inventory documents all 74 Dependabot alerts flagged across the repository's three dependency trees:
1. **Node.js Backend & Root Orchestrator (`package-lock.json`)**: 44 alerts
2. **Frontend React/Vite Client (`frontend/package-lock.json`)**: 22 alerts
3. **Python AI/ML Sidecar (`sidecars/python/requirements.txt`)**: 8 alerts

### Severity Breakdown
| Severity | Total | Critical Concerns | Primary Mitigation |
| :--- | :--- | :--- | :--- |
| **Critical** | 2 | `sentence-transformers` (CVE-2026-68770), `proxy-addr` (CVE-2026-90711) | Upgrade `sentence-transformers` to >= 5.6.0; `proxy-addr` unreachable in desktop setup |
| **High** | 25 | `torch` (CVE-2026-24747), `python-multipart` DoS, `undici` TLS/cache, dev-only `tar-fs`/`extract-zip` | Upgrade `torch` to >= 2.10.0; upgrade `python-multipart` to >= 0.0.30 |
| **Medium** | 35 | Node/Vite build-time & transitive libraries | Standard `npm audit fix` / targeted package bumps |
| **Low** | 12 | Edge-case protocol/parser advisories | Periodic dependency hygiene |

---

## Complete 74-Alert Itemized Register

| # | Package | Ecosystem | Severity | Advisory / CVE | Relationship | Manifest | In Installer? | Patched Version | Classification |
|---|---|---|---|---|---|---|---|---|---|
| 74 | `torch` | pip | **HIGH** | CVE-2026-24747 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `2.10.0` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 73 | `sentence-transformers` | pip | **CRITICAL** | CVE-2026-68770 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `5.6.0` | Critical Risk Unpatched Dependency (CVE-2026-68770) |
| 72 | `torch` | pip | **LOW** | CVE-2025-3000 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `2.13.0` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 71 | `python-multipart` | pip | **HIGH** | CVE-2026-53539 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.30` | Medium Risk / Local DoS |
| 70 | `python-multipart` | pip | **LOW** | CVE-2026-53540 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.31` | Medium Risk / Local DoS |
| 69 | `python-multipart` | pip | **LOW** | CVE-2026-53538 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.30` | Medium Risk / Local DoS |
| 68 | `python-multipart` | pip | **LOW** | CVE-2026-53537 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.30` | Medium Risk / Local DoS |
| 67 | `torch` | pip | **LOW** | CVE-2025-3001 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `2.10.0` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 66 | `torch` | pip | **MEDIUM** | CVE-2025-2999 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `2.9.1` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 65 | `torch` | pip | **LOW** | CVE-2025-2149 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `None` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 64 | `torch` | pip | **MEDIUM** | CVE-2025-2998 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `None` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 63 | `torch` | pip | **LOW** | CVE-2025-2148 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `None` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 62 | `python-multipart` | pip | **HIGH** | CVE-2026-42561 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.27` | Medium Risk / Local DoS |
| 61 | `python-multipart` | pip | **MEDIUM** | CVE-2026-40347 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.26` | Medium Risk / Local DoS |
| 60 | `python-multipart` | pip | **HIGH** | CVE-2026-24486 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.22` | Medium Risk / Local DoS |
| 59 | `torch` | pip | **MEDIUM** | CVE-2025-3730 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `2.8.0` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 58 | `torch` | pip | **LOW** | CVE-2025-2953 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `2.7.1-rc1` | High Risk Unpatched Dependency (CVE-2026-24747) |
| 57 | `python-multipart` | pip | **HIGH** | CVE-2024-53981 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.18` | Medium Risk / Local DoS |
| 56 | `python-multipart` | pip | **HIGH** | CVE-2024-24762 | Direct | `sidecars/python/requirements.txt` | Yes (Sidecar/Dist) | `0.0.7` | Medium Risk / Local DoS |
| 55 | `undici` | npm | **HIGH** | CVE-2026-109829 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 54 | `undici` | npm | **HIGH** | CVE-2026-109383 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 53 | `undici` | npm | **HIGH** | CVE-2026-87018 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 52 | `undici` | npm | **MEDIUM** | CVE-2026-109932 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 51 | `undici` | npm | **MEDIUM** | CVE-2026-87029 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 50 | `qs` | npm | **MEDIUM** | CVE-2026-110816 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `6.16.1` | Dependency Hygiene / Patch Available |
| 49 | `undici` | npm | **HIGH** | CVE-2026-109869 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 48 | `undici` | npm | **MEDIUM** | CVE-2026-109924 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 47 | `undici` | npm | **MEDIUM** | CVE-2026-109841 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `7.30.1` | Architectural False Positive / Unreachable |
| 46 | `multer` | npm | **HIGH** | CVE-2026-113872 | Direct | `package-lock.json` | Yes (Sidecar/Dist) | `2.5.0` | Low Risk / Local DoS Only |
| 45 | `body-parser` | npm | **HIGH** | CVE-2026-111181 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `2.4.0` | Low Risk / Local DoS Only |
| 44 | `proxy-addr` | npm | **CRITICAL** | CVE-2026-90711 | Transitive | `package-lock.json` | Yes (Sidecar/Dist) | `2.0.8` | Architectural False Positive / Unreachable |
| 43 | `multer` | npm | **MEDIUM** | CVE-2026-88932 | Direct | `package-lock.json` | Yes (Sidecar/Dist) | `2.4.0` | Low Risk / Local DoS Only |
| 42 | `extract-zip` | npm | **HIGH** | CVE-2026-19693 | Transitive | `package-lock.json` | No (Dev/Build) | `None` | Dev-Only / Non-Production False Positive |
| 41 | `extract-zip` | npm | **HIGH** | CVE-2026-56876 | Transitive | `package-lock.json` | No (Dev/Build) | `None` | Dev-Only / Non-Production False Positive |
| 40 | `ws` | npm | **HIGH** | CVE-2026-48779 | Transitive | `package-lock.json` | No (Dev/Build) | `8.21.0` | Dev-Only / Non-Production False Positive |
| 39 | `ws` | npm | **MEDIUM** | CVE-2026-45736 | Transitive | `package-lock.json` | No (Dev/Build) | `8.20.1` | Dev-Only / Non-Production False Positive |
| 38 | `tar-fs` | npm | **HIGH** | CVE-2025-59343 | Transitive | `package-lock.json` | No (Dev/Build) | `2.1.4` | Dev-Only / Non-Production False Positive |
| 37 | `tar-fs` | npm | **HIGH** | CVE-2025-48387 | Transitive | `package-lock.json` | No (Dev/Build) | `2.1.3` | Dev-Only / Non-Production False Positive |
| 36 | `tar-fs` | npm | **HIGH** | CVE-2024-12905 | Transitive | `package-lock.json` | No (Dev/Build) | `2.1.2` | Dev-Only / Non-Production False Positive |
| 35 | `ws` | npm | **HIGH** | CVE-2024-37890 | Transitive | `package-lock.json` | No (Dev/Build) | `8.17.1` | Dev-Only / Non-Production False Positive |
| 34 | `postcss-selector-parser` | npm | **MEDIUM** | CVE-2026-104844 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `6.1.5` | Frontend Build Dependency / Low Risk |
| 33 | `vite` | npm | **LOW** | CVE-2026-112945 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.4` | Dev-Only / Non-Production False Positive |
| 32 | `vite` | npm | **MEDIUM** | CVE-2026-112946 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.4` | Dev-Only / Non-Production False Positive |
| 31 | `vite` | npm | **MEDIUM** | CVE-2026-112944 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.4` | Dev-Only / Non-Production False Positive |
| 30 | `vite` | npm | **HIGH** | CVE-2026-112942 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.4` | Dev-Only / Non-Production False Positive |
| 29 | `vite` | npm | **MEDIUM** | CVE-2026-112943 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.4` | Dev-Only / Non-Production False Positive |
| 28 | `katex` | npm | **LOW** | CVE-2026-103923 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `0.18.2` | Frontend Build Dependency / Low Risk |
| 27 | `source-map-js` | npm | **HIGH** | CVE-2026-93749 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `1.2.2` | Dev-Only / Non-Production False Positive |
| 26 | `@remix-run/router` | npm | **MEDIUM** | CVE-2026-40181 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `1.23.3` | Low Risk Runtime / Dependency Update Needed |
| 25 | `postcss` | npm | **MEDIUM** | CVE-2026-69153 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `8.5.23` | Dev-Only / Non-Production False Positive |
| 24 | `postcss` | npm | **HIGH** | CVE-2026-73646 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `8.5.18` | Dev-Only / Non-Production False Positive |
| 23 | `react-router` | npm | **MEDIUM** | CVE-2026-53669 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `7.18.0` | Low Risk Runtime / Dependency Update Needed |
| 22 | `react-router` | npm | **MEDIUM** | CVE-2026-53666 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `7.18.0` | Low Risk Runtime / Dependency Update Needed |
| 21 | `postcss` | npm | **HIGH** | CVE-2026-45623 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `8.5.12` | Dev-Only / Non-Production False Positive |
| 20 | `vite` | npm | **MEDIUM** | CVE-2026-53632 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.3` | Dev-Only / Non-Production False Positive |
| 19 | `vite` | npm | **MEDIUM** | CVE-2026-53571 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.3` | Dev-Only / Non-Production False Positive |
| 18 | `react-router` | npm | **MEDIUM** | CVE-2026-40181 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `6.30.4` | Low Risk Runtime / Dependency Update Needed |
| 17 | `vite` | npm | **HIGH** | CVE-2024-52011 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.9` | Dev-Only / Non-Production False Positive |
| 16 | `postcss` | npm | **MEDIUM** | CVE-2026-41305 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `8.5.10` | Dev-Only / Non-Production False Positive |
| 15 | `vite` | npm | **MEDIUM** | CVE-2026-39365 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `6.4.2` | Dev-Only / Non-Production False Positive |
| 14 | `@remix-run/router` | npm | **HIGH** | CVE-2026-22029 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `1.23.2` | Low Risk Runtime / Dependency Update Needed |
| 13 | `react-router` | npm | **MEDIUM** | CVE-2025-68470 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `6.30.2` | Low Risk Runtime / Dependency Update Needed |
| 12 | `vite` | npm | **LOW** | CVE-2025-58751 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.20` | Dev-Only / Non-Production False Positive |
| 11 | `vite` | npm | **LOW** | CVE-2025-58752 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.20` | Dev-Only / Non-Production False Positive |
| 10 | `vite` | npm | **MEDIUM** | CVE-2025-46565 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.19` | Dev-Only / Non-Production False Positive |
| 9 | `vite` | npm | **MEDIUM** | CVE-2025-32395 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.18` | Dev-Only / Non-Production False Positive |
| 8 | `vite` | npm | **MEDIUM** | CVE-2025-31486 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.17` | Dev-Only / Non-Production False Positive |
| 7 | `vite` | npm | **MEDIUM** | CVE-2025-31125 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.16` | Dev-Only / Non-Production False Positive |
| 6 | `vite` | npm | **MEDIUM** | CVE-2025-30208 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.15` | Dev-Only / Non-Production False Positive |
| 5 | `esbuild` | npm | **MEDIUM** | GHSA-67mh-4wv8-2f99 | Transitive | `frontend/package-lock.json` | No (Dev/Build) | `0.25.0` | Frontend Build Dependency / Low Risk |
| 4 | `vite` | npm | **MEDIUM** | CVE-2025-24010 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.4.12` | Dev-Only / Non-Production False Positive |
| 3 | `vite` | npm | **MEDIUM** | CVE-2024-45811 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.1.8` | Dev-Only / Non-Production False Positive |
| 2 | `vite` | npm | **MEDIUM** | CVE-2024-45812 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.1.8` | Dev-Only / Non-Production False Positive |
| 1 | `vite` | npm | **MEDIUM** | CVE-2024-31207 | Direct | `frontend/package-lock.json` | No (Dev/Build) | `5.1.7` | Dev-Only / Non-Production False Positive |

---

## Forensic Item Analysis by Component

### Tier: Python ML Sidecar Runtime (19 Alerts)

#### Alert #74: `torch` — PyTorch Vulnerable to Remote Code Execution via Untrusted Checkpoint Files
- **Advisory ID:** GHSA-63cw-57p8-fm3p (CVE-2026-24747)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 2.10.0` | **First Patched:** `2.10.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Update `torch` to `2.10.0` or later.

#### Alert #73: `sentence-transformers` — sentence-transformers local model loading bypasses trust_remote_code and executes custom Python
- **Advisory ID:** GHSA-jhr6-gm9c-rqjv (CVE-2026-68770)
- **Severity:** `CRITICAL`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 5.6.0` | **First Patched:** `5.6.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** Local model loading logic flaw in sentence_transformers/util/misc.py bypasses trust_remote_code=False whenever directory exists on disk. Local model directory contains modules.json which could be modified by local processes. Remediated in sentence-transformers >= 5.6.0.
- **Recommended Remediation:** Update `sentence-transformers` to `5.6.0` or later.

#### Alert #72: `torch` — PyTorch is vulnerable to memory corruption through its torch.jit.script function
- **Advisory ID:** GHSA-rrmf-rvhw-rf47 (CVE-2025-3000)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `<= 2.12.1` | **First Patched:** `2.13.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Update `torch` to `2.13.0` or later.

#### Alert #71: `python-multipart` — python-multipart: Quadratic-time querystring parsing with semicolon separators causes CPU denial of service
- **Advisory ID:** GHSA-5rvq-cxj2-64vf (CVE-2026-53539)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.30` | **First Patched:** `0.0.30`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.30` or later.

#### Alert #70: `python-multipart` — python-multipart: Negative Content-Length in parse_form buffers the entire body in memory
- **Advisory ID:** GHSA-v9pg-7xvm-68hf (CVE-2026-53540)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.31` | **First Patched:** `0.0.31`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.31` or later.

#### Alert #69: `python-multipart` — python-multipart: Semicolon treated as querystring field separator enables parameter smuggling
- **Advisory ID:** GHSA-6jv3-5f52-599m (CVE-2026-53538)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.30` | **First Patched:** `0.0.30`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.30` or later.

#### Alert #68: `python-multipart` — python-multipart: Content-Disposition parameter smuggling via RFC 2231/5987 extended parameters
- **Advisory ID:** GHSA-vffw-93wf-4j4q (CVE-2026-53537)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.30` | **First Patched:** `0.0.30`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.30` or later.

#### Alert #67: `torch` — PyTorch is vulnerable to memory corruption through its torch.lstm_cell function
- **Advisory ID:** GHSA-qfhq-4f3w-5fph (CVE-2025-3001)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 2.10.0` | **First Patched:** `2.10.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Update `torch` to `2.10.0` or later.

#### Alert #66: `torch` — PyTorch is vulnerable to memory corruption through its unpack_sequence function
- **Advisory ID:** GHSA-vgrw-7cvw-pwgx (CVE-2025-2999)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 2.9.1` | **First Patched:** `2.9.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Update `torch` to `2.9.1` or later.

#### Alert #65: `torch` — PyTorch: Manipulation of the argument scale/zero_point leads to improper initialization via Quantized Sigmoid Module
- **Advisory ID:** GHSA-x3gm-94wq-g975 (CVE-2025-2149)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `<= 2.6.0` | **First Patched:** `None`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Vendor patch pending; restrict untrusted inputs.

#### Alert #64: `torch` — PyTorch is Vulnerable to Memory Consumption through pad_packed_sequence Function
- **Advisory ID:** GHSA-f4hp-rmr7-r7v8 (CVE-2025-2998)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `<= 2.6.0` | **First Patched:** `None`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Vendor patch pending; restrict untrusted inputs.

#### Alert #63: `torch` — PyTorch Tuple Handler is Vulnerable to Memory Corruption through Manipulation of None Argument
- **Advisory ID:** GHSA-c678-jfcj-6jmf (CVE-2025-2148)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `<= 2.6.0` | **First Patched:** `None`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Vendor patch pending; restrict untrusted inputs.

#### Alert #62: `python-multipart` — python-multipart has Denial of Service via unbounded multipart part headers
- **Advisory ID:** GHSA-pp6c-gr5w-3c5g (CVE-2026-42561)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.27` | **First Patched:** `0.0.27`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.27` or later.

#### Alert #61: `python-multipart` — python-multipart affected by Denial of Service via large multipart preamble or epilogue data
- **Advisory ID:** GHSA-mj87-hwqh-73pj (CVE-2026-40347)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.26` | **First Patched:** `0.0.26`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.26` or later.

#### Alert #60: `python-multipart` — Python-Multipart has Arbitrary File Write via Non-Default Configuration
- **Advisory ID:** GHSA-wp53-j4wj-2cfg (CVE-2026-24486)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.22` | **First Patched:** `0.0.22`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.22` or later.

#### Alert #59: `torch` — PyTorch Improper Resource Shutdown or Release vulnerability
- **Advisory ID:** GHSA-887c-mr87-cxwp (CVE-2025-3730)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `<= 2.7.1` | **First Patched:** `2.8.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Update `torch` to `2.8.0` or later.

#### Alert #58: `torch` — PyTorch susceptible to local Denial of Service
- **Advisory ID:** GHSA-3749-ghw9-m3mg (CVE-2025-2953)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 2.7.1-rc1` | **First Patched:** `2.7.1-rc1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** PyTorch weights_only unpickler memory corruption flaw. While weights are locally sourced from official Kokoro, lack of file integrity checks creates risk if local files are altered. Remediated in torch >= 2.10.0.
- **Recommended Remediation:** Update `torch` to `2.7.1-rc1` or later.

#### Alert #57: `python-multipart` — Denial of service (DoS) via deformation `multipart/form-data` boundary
- **Advisory ID:** GHSA-59g5-xgcq-4qw3 (CVE-2024-53981)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `< 0.0.18` | **First Patched:** `0.0.18`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.18` or later.

#### Alert #56: `python-multipart` — python-multipart vulnerable to Content-Type Header ReDoS
- **Advisory ID:** GHSA-2jv5-9r88-3w3p (CVE-2024-24762)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `sidecars/python/requirements.txt`
- **Vulnerable Range:** `<= 0.0.6` | **First Patched:** `0.0.7`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Python ML Sidecar Runtime
- **Forensic Assessment & Evidence:** FastAPI UploadFile endpoints in ocr/routes.py and parse/routes.py use python-multipart. DoS vulnerability against multipart parser. Remediated in python-multipart >= 0.0.30.
- **Recommended Remediation:** Update `python-multipart` to `0.0.7` or later.

### Tier: Backend Runtime (Transitive via Node fetch) (8 Alerts)

#### Alert #55: `undici` — undici vulnerable to cache poisoning and cross-user disclosure via cross-origin redirects
- **Advisory ID:** GHSA-4w7w-5xrc-9h26 (CVE-2026-109829)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.0.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #54: `undici` — undici vulnerable to TLS certificate validation bypass via dropped tls options in Agent
- **Advisory ID:** GHSA-2m5v-pch6-74hx (CVE-2026-109383)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.0.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #53: `undici` — undici vulnerable to Denial of Service via TLS 1.2 certificate-changing renegotiation
- **Advisory ID:** GHSA-qrxm-fqhp-jwvm (CVE-2026-87018)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.0.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #52: `undici` — undici WebSocket client vulnerable to Denial of Service via unbounded automatic PONG buffering
- **Advisory ID:** GHSA-9fm3-97px-rjf6 (CVE-2026-109932)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.0.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #51: `undici` — undici vulnerable to Denial of Service via unhandled exception in the decompress interceptor
- **Advisory ID:** GHSA-55wc-3pg8-jfw6 (CVE-2026-87029)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.15.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #49: `undici` — undici vulnerable to Denial of Service via unhandled error on excessive content-encodings
- **Advisory ID:** GHSA-xjj6-f33q-929c (CVE-2026-109869)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.18.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #48: `undici` — undici vulnerable to Denial of Service via unbounded memory in the EventSource parser
- **Advisory ID:** GHSA-4qv8-2p9j-2685 (CVE-2026-109924)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.0.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

#### Alert #47: `undici` — undici vulnerable to Subresource Integrity bypass via non-lowercase algorithm tokens
- **Advisory ID:** GHSA-f72v-6w45-q472 (CVE-2026-109841)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 7.15.0, < 7.30.1` | **First Patched:** `7.30.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Node fetch)
- **Forensic Assessment & Evidence:** Cross-user cache poisoning and HTTP header issues in undici. Omnitome is single-user local desktop software; multi-user cache poisoning cannot occur.
- **Recommended Remediation:** Update `undici` to `7.30.1` or later.

### Tier: Backend Node Runtime (1 Alerts)

#### Alert #50: `qs` — qs : qs.stringify with arrayFormat 'comma' throws an uncaught TypeError on array elements carrying a non-callable own toString (DoS)
- **Advisory ID:** GHSA-6pq8-jhm2-2vcw (CVE-2026-110816)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 6.7.0, <= 6.16.0` | **First Patched:** `6.16.1`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Node Runtime
- **Forensic Assessment & Evidence:** Standard Node.js runtime dependency. Recommended to upgrade via npm update.
- **Recommended Remediation:** Update `qs` to `6.16.1` or later.

### Tier: Backend Runtime (Direct/Transitive Express) (3 Alerts)

#### Alert #46: `multer` — multer vulnerable to Denial of Service via a pending asynchronous fileFilter after request teardown
- **Advisory ID:** GHSA-q7qp-rjg7-q26m (CVE-2026-113872)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `package-lock.json`
- **Vulnerable Range:** `>= 2.0.0, < 2.5.0` | **First Patched:** `2.5.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Direct/Transitive Express)
- **Forensic Assessment & Evidence:** Trailing byte / fileFilter DoS vectors. In local desktop software, client is localhost. Patched versions available via npm update.
- **Recommended Remediation:** Update `multer` to `2.5.0` or later.

#### Alert #45: `body-parser` — body-parser vulnerable to Denial of Service via trailing bytes after a compressed request body
- **Advisory ID:** GHSA-27r3-jxmh-p828 (CVE-2026-111181)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 2.0.0, < 2.4.0` | **First Patched:** `2.4.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Direct/Transitive Express)
- **Forensic Assessment & Evidence:** Trailing byte / fileFilter DoS vectors. In local desktop software, client is localhost. Patched versions available via npm update.
- **Recommended Remediation:** Update `body-parser` to `2.4.0` or later.

#### Alert #43: `multer` — multer vulnerable to Denial of Service via orphaned disk writes on aborted uploads
- **Advisory ID:** GHSA-3pph-fpjx-jg34 (CVE-2026-88932)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `package-lock.json`
- **Vulnerable Range:** `>= 2.2.0, < 2.4.0` | **First Patched:** `2.4.0`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Direct/Transitive Express)
- **Forensic Assessment & Evidence:** Trailing byte / fileFilter DoS vectors. In local desktop software, client is localhost. Patched versions available via npm update.
- **Recommended Remediation:** Update `multer` to `2.4.0` or later.

### Tier: Backend Runtime (Transitive via Express) (1 Alerts)

#### Alert #44: `proxy-addr` — proxy-addr vulnerable to IP spoofing via IPv4-mapped IPv6 trust subnet
- **Advisory ID:** GHSA-jqcg-44mw-7w3h (CVE-2026-90711)
- **Severity:** `CRITICAL`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 1.1.0, < 2.0.8` | **First Patched:** `2.0.8`
- **Packaged in Windows Installer:** YES
- **Vulnerable Code Path / Usage:** Backend Runtime (Transitive via Express)
- **Forensic Assessment & Evidence:** Express IP spoofing vulnerability under reverse-proxy configuration. Omnitome runs as a local-only daemon on 127.0.0.1 without reverse proxies, subnet trust rules, or IP-based authorization.
- **Recommended Remediation:** Update `proxy-addr` to `2.0.8` or later.

### Tier: Dev / Studio Tooling (8 Alerts)

#### Alert #42: `extract-zip` — extract-zip allows arbitrary file writes through symlink archive entries
- **Advisory ID:** GHSA-7pqw-9j4j-h8q3 (CVE-2026-19693)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `<= 2.0.1` | **First Patched:** `None`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** extract-zip is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Vendor patch pending; restrict untrusted inputs.

#### Alert #41: `extract-zip` — extract-zip unvalidated symlink path traversal
- **Advisory ID:** GHSA-jmr9-qjv8-65gv (CVE-2026-56876)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `<= 2.0.1` | **First Patched:** `None`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** extract-zip is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Vendor patch pending; restrict untrusted inputs.

#### Alert #40: `ws` — ws: Memory exhaustion DoS from tiny fragments and data chunks
- **Advisory ID:** GHSA-96hv-2xvq-fx4p (CVE-2026-48779)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 8.0.0, < 8.21.0` | **First Patched:** `8.21.0`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** ws is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Update `ws` to `8.21.0` or later.

#### Alert #39: `ws` — ws: Uninitialized memory disclosure
- **Advisory ID:** GHSA-58qx-3vcg-4xpx (CVE-2026-45736)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 8.0.0, < 8.20.1` | **First Patched:** `8.20.1`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** ws is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Update `ws` to `8.20.1` or later.

#### Alert #38: `tar-fs` — tar-fs has a symlink validation bypass if destination directory is predictable with a specific tarball
- **Advisory ID:** GHSA-vj76-c3g6-qr5v (CVE-2025-59343)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 2.0.0, < 2.1.4` | **First Patched:** `2.1.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** tar-fs is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Update `tar-fs` to `2.1.4` or later.

#### Alert #37: `tar-fs` — tar-fs can extract outside the specified dir with a specific tarball
- **Advisory ID:** GHSA-8cj5-5rvv-wf4v (CVE-2025-48387)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 2.0.0, < 2.1.3` | **First Patched:** `2.1.3`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** tar-fs is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Update `tar-fs` to `2.1.3` or later.

#### Alert #36: `tar-fs` — tar-fs Vulnerable to Link Following and Path Traversal via Extracting a Crafted tar File
- **Advisory ID:** GHSA-pq67-2wwv-3xjx (CVE-2024-12905)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 2.0.0, < 2.1.2` | **First Patched:** `2.1.2`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** tar-fs is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Update `tar-fs` to `2.1.2` or later.

#### Alert #35: `ws` — ws affected by a DoS when handling a request with many HTTP headers
- **Advisory ID:** GHSA-3h5v-q93c-6h6q (CVE-2024-37890)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `package-lock.json`
- **Vulnerable Range:** `>= 8.0.0, < 8.17.1` | **First Patched:** `8.17.1`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Dev / Studio Tooling
- **Forensic Assessment & Evidence:** ws is a transitive dependency of puppeteer-screen-recorder (used exclusively for studio video generation in _studio/). Excluded from desktop installer.
- **Recommended Remediation:** Update `ws` to `8.17.1` or later.

### Tier: Frontend (Build/Dev) (34 Alerts)

#### Alert #34: `postcss-selector-parser` — PostCSS: Quadratic complexity in flat selector parsing allows CPU exhaustion
- **Advisory ID:** GHSA-rj75-hqrm-r3gf (CVE-2026-104844)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `< 6.1.5` | **First Patched:** `6.1.5`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Transitive frontend build dependency. Pre-compiled during build.
- **Recommended Remediation:** Update `postcss-selector-parser` to `6.1.5` or later.

#### Alert #33: `vite` — Vite: Cross origin script execution through Vite custom middleware
- **Advisory ID:** GHSA-9jrq-w75r-8gcw (CVE-2026-112945)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.3` | **First Patched:** `6.4.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.4` or later.

#### Alert #32: `vite` — Vite: Files outside `server.fs.allow` could be served when project paths match system paths on non-Windows machine
- **Advisory ID:** GHSA-rq7h-c2jc-7f22 (CVE-2026-112946)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.3` | **First Patched:** `6.4.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.4` or later.

#### Alert #31: `vite` — launch-editor: Editor CLI flag misinterpretation causing arbitrary command execution for some editors
- **Advisory ID:** GHSA-r825-w8mm-5pxw (CVE-2026-112944)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.3` | **First Patched:** `6.4.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.4` or later.

#### Alert #30: `vite` — launch-editor: `fs.existsSync()` existence check can be bypassed on Windows
- **Advisory ID:** GHSA-587h-7wmj-h5pm (CVE-2026-112942)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.3` | **First Patched:** `6.4.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.4` or later.

#### Alert #29: `vite` — launch-editor: escapeCmdArgs() omits % escaping on Windows
- **Advisory ID:** GHSA-698v-4r3x-3vcv (CVE-2026-112943)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.3` | **First Patched:** `6.4.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.4` or later.

#### Alert #28: `katex` — KaTeX: Existing prototype pollution can bypass trust restrictions
- **Advisory ID:** GHSA-238p-pmpm-9mq7 (CVE-2026-103923)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 0.11.0, < 0.18.2` | **First Patched:** `0.18.2`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Transitive frontend build dependency. Pre-compiled during build.
- **Recommended Remediation:** Update `katex` to `0.18.2` or later.

#### Alert #27: `source-map-js` — source-map-js allows event-loop denial of service through indexed source-map section offsets
- **Advisory ID:** GHSA-68fv-2mgg-jv7q (CVE-2026-93749)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 1.0.0, < 1.2.2` | **First Patched:** `1.2.2`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** source-map-js is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `source-map-js` to `1.2.2` or later.

#### Alert #26: `@remix-run/router` — React Router's same-origin redirect with path starting // causes open redirect via protocol-relative URL reinterpretation
- **Advisory ID:** GHSA-2j2x-hqr9-3h42 (CVE-2026-40181)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 1.3.0, < 1.23.3` | **First Patched:** `1.23.3`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Compiled into frontend bundle. Client-side routing in Tauri does not perform open server redirects, but upgrading react-router is standard practice.
- **Recommended Remediation:** Update `@remix-run/router` to `1.23.3` or later.

#### Alert #25: `postcss` — PostCSS: incomplete fix of GHSA-6g55-p6wh-862q — attacker-controlled sourceMappingURL reads arbitrary .map files when `from` is unset
- **Advisory ID:** GHSA-fxqj-rqcc-2cmp (CVE-2026-69153)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 8.5.22` | **First Patched:** `8.5.23`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** postcss is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `postcss` to `8.5.23` or later.

#### Alert #24: `postcss` — PostCSS: Path Traversal in Previous Source Map Auto-Loading (sourceMappingURL) leads to Arbitrary .map File Disclosure
- **Advisory ID:** GHSA-r28c-9q8g-f849 (CVE-2026-73646)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 8.5.17` | **First Patched:** `8.5.18`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** postcss is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `postcss` to `8.5.18` or later.

#### Alert #23: `react-router` — React Router: Open redirect via backslash in <Link> and useNavigate (CVE-2025-68470 bypass)
- **Advisory ID:** GHSA-wrjc-x8rr-h8h6 (CVE-2026-53669)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 6.0.0, < 7.18.0` | **First Patched:** `7.18.0`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Compiled into frontend bundle. Client-side routing in Tauri does not perform open server redirects, but upgrading react-router is standard practice.
- **Recommended Remediation:** Update `react-router` to `7.18.0` or later.

#### Alert #22: `react-router` — React Router: Arbitrary Constructor Injection via deserializeErrors() in React Router SSR Hydration
- **Advisory ID:** GHSA-337j-9hxr-rhxg (CVE-2026-53666)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 6.4.0, < 7.18.0` | **First Patched:** `7.18.0`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Compiled into frontend bundle. Client-side routing in Tauri does not perform open server redirects, but upgrading react-router is standard practice.
- **Recommended Remediation:** Update `react-router` to `7.18.0` or later.

#### Alert #21: `postcss` — PostCSS: Arbitrary file read and information disclosure via attacker-controlled sourceMappingURL in CSS comments
- **Advisory ID:** GHSA-6g55-p6wh-862q (CVE-2026-45623)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 8.5.11` | **First Patched:** `8.5.12`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** postcss is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `postcss` to `8.5.12` or later.

#### Alert #20: `vite` — launch-editor: NTLMv2 hash disclosure via UNC path handling on Windows
- **Advisory ID:** GHSA-v6wh-96g9-6wx3 (CVE-2026-53632)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.2` | **First Patched:** `6.4.3`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.3` or later.

#### Alert #19: `vite` — vite: `server.fs.deny` bypass on Windows alternate paths
- **Advisory ID:** GHSA-fx2h-pf6j-xcff (CVE-2026-53571)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.2` | **First Patched:** `6.4.3`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.3` or later.

#### Alert #18: `react-router` — React Router's same-origin redirect with path starting // causes open redirect via protocol-relative URL reinterpretation
- **Advisory ID:** GHSA-2j2x-hqr9-3h42 (CVE-2026-40181)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 6.7.0, < 6.30.4` | **First Patched:** `6.30.4`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Compiled into frontend bundle. Client-side routing in Tauri does not perform open server redirects, but upgrading react-router is standard practice.
- **Recommended Remediation:** Update `react-router` to `6.30.4` or later.

#### Alert #17: `vite` — launch-editor vulnerable to command injection via the crafted request on Windows
- **Advisory ID:** GHSA-c27g-q93r-2cwf (CVE-2024-52011)
- **Severity:** `HIGH`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 5.4.8` | **First Patched:** `5.4.9`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.9` or later.

#### Alert #16: `postcss` — PostCSS has XSS via Unescaped </style> in its CSS Stringify Output
- **Advisory ID:** GHSA-qx2v-qp2m-jg93 (CVE-2026-41305)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `< 8.5.10` | **First Patched:** `8.5.10`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** postcss is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `postcss` to `8.5.10` or later.

#### Alert #15: `vite` — Vite Vulnerable to Path Traversal in Optimized Deps `.map` Handling
- **Advisory ID:** GHSA-4w7w-66w2-5vf9 (CVE-2026-39365)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 6.4.1` | **First Patched:** `6.4.2`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `6.4.2` or later.

#### Alert #14: `@remix-run/router` — React Router vulnerable to XSS via Open Redirects
- **Advisory ID:** GHSA-2w69-qvjg-hvjx (CVE-2026-22029)
- **Severity:** `HIGH`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 1.23.1` | **First Patched:** `1.23.2`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Compiled into frontend bundle. Client-side routing in Tauri does not perform open server redirects, but upgrading react-router is standard practice.
- **Recommended Remediation:** Update `@remix-run/router` to `1.23.2` or later.

#### Alert #13: `react-router` — React Router has unexpected external redirect via untrusted paths
- **Advisory ID:** GHSA-9jcx-v3wj-wh4m (CVE-2025-68470)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 6.0.0, < 6.30.2` | **First Patched:** `6.30.2`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Compiled into frontend bundle. Client-side routing in Tauri does not perform open server redirects, but upgrading react-router is standard practice.
- **Recommended Remediation:** Update `react-router` to `6.30.2` or later.

#### Alert #12: `vite` — Vite middleware may serve files starting with the same name with the public directory
- **Advisory ID:** GHSA-g4jq-h2w9-997c (CVE-2025-58751)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 5.4.19` | **First Patched:** `5.4.20`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.20` or later.

#### Alert #11: `vite` — Vite's `server.fs` settings were not applied to HTML files
- **Advisory ID:** GHSA-jqfw-vq24-v9c3 (CVE-2025-58752)
- **Severity:** `LOW`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 5.4.19` | **First Patched:** `5.4.20`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.20` or later.

#### Alert #10: `vite` — Vite's server.fs.deny bypassed with /. for files under project root
- **Advisory ID:** GHSA-859w-5945-r5v3 (CVE-2025-46565)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, <= 5.4.18` | **First Patched:** `5.4.19`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.19` or later.

#### Alert #9: `vite` — Vite has an `server.fs.deny` bypass with an invalid `request-target`
- **Advisory ID:** GHSA-356w-63v5-8wf4 (CVE-2025-32395)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, < 5.4.18` | **First Patched:** `5.4.18`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.18` or later.

#### Alert #8: `vite` — Vite allows server.fs.deny to be bypassed with .svg or relative paths
- **Advisory ID:** GHSA-xcj6-pq6g-qj4x (CVE-2025-31486)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, < 5.4.17` | **First Patched:** `5.4.17`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.17` or later.

#### Alert #7: `vite` — Vite has a `server.fs.deny` bypassed for `inline` and `raw` with `?import` query
- **Advisory ID:** GHSA-4r4m-qw57-chr8 (CVE-2025-31125)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, < 5.4.16` | **First Patched:** `5.4.16`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.16` or later.

#### Alert #6: `vite` — Vite bypasses server.fs.deny when using ?raw??
- **Advisory ID:** GHSA-x574-m823-4x7w (CVE-2025-30208)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, < 5.4.15` | **First Patched:** `5.4.15`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.15` or later.

#### Alert #5: `esbuild` — esbuild enables any website to send any requests to the development server and read the response
- **Advisory ID:** GHSA-67mh-4wv8-2f99 (N/A)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Transitive dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `<= 0.24.2` | **First Patched:** `0.25.0`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** Transitive frontend build dependency. Pre-compiled during build.
- **Recommended Remediation:** Update `esbuild` to `0.25.0` or later.

#### Alert #4: `vite` — Websites were able to send any requests to the development server and read the response in vite
- **Advisory ID:** GHSA-vg6x-rcgg-rjx6 (CVE-2025-24010)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, <= 5.4.11` | **First Patched:** `5.4.12`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.4.12` or later.

#### Alert #3: `vite` — Vite's `server.fs.deny` is bypassed when using `?import&raw`
- **Advisory ID:** GHSA-9cwx-2883-4wfx (CVE-2024-45811)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, <= 5.1.7` | **First Patched:** `5.1.8`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.1.8` or later.

#### Alert #2: `vite` — Vite DOM Clobbering gadget found in vite bundled scripts that leads to XSS
- **Advisory ID:** GHSA-64vr-g452-qvp3 (CVE-2024-45812)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.0.0, < 5.1.8` | **First Patched:** `5.1.8`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.1.8` or later.

#### Alert #1: `vite` — Vite's `server.fs.deny` did not deny requests for patterns with directories.
- **Advisory ID:** GHSA-8jhw-289h-jh2g (CVE-2024-31207)
- **Severity:** `MEDIUM`
- **Dependency Scope:** Direct dependency in `frontend/package-lock.json`
- **Vulnerable Range:** `>= 5.1.0, <= 5.1.6` | **First Patched:** `5.1.7`
- **Packaged in Windows Installer:** NO
- **Vulnerable Code Path / Usage:** Frontend (Build/Dev)
- **Forensic Assessment & Evidence:** vite is a build-time dev tool in frontend/. Only static HTML/CSS/JS is bundled into Tauri WebView2. Vite dev-server does not run in production.
- **Recommended Remediation:** Update `vite` to `5.1.7` or later.

