# Security Policy — Omnitome

Omnitome is committed to ensuring the safety, privacy, and integrity of our users and their personal reading libraries. Because Omnitome operates as a local-first, privacy-preserving desktop application that runs neural models on-device, our security model differs fundamentally from public multi-tenant cloud services.

---

## 1. Supported Versions

We actively provide security fixes, dependency patches, and vulnerability remediations for the following release lines:

| Version | Supported | Status |
| :--- | :---: | :--- |
| **1.1.x** | :white_check_mark: | **Current Stable** (Active security & patch maintenance) |
| **1.0.x** | :warning: | **Maintenance** (Critical security vulnerabilities only) |
| **< 1.0** | :x: | **End of Life** (Unsupported) |

---

## 2. Core Security & Privacy Invariants

When evaluating vulnerabilities in Omnitome, our architecture enforces the following non-negotiable boundaries:

1. **Local-First & Zero Egress:**  
   - All document text, SQLite databases, vector embeddings, and generated speech files remain strictly on the user's local machine (`127.0.0.1`).  
   - Omnitome collects **zero telemetry** and performs **zero outbound egress** of document content or model weights.
2. **Neural Checkpoint Integrity Gate:**  
   - Deserialization of model weights (PyTorch, Kokoro-82M, BGE-M3) is gatekept by cryptographic SHA-256 validation against authenticated Git LFS OIDs before execution (`sidecars/python/security/integrity.py`).  
   - Safetensors is strictly prioritized over legacy binary pickle formats.
3. **Multi-Process Containment:**  
   - **Tauri Shell (Rust):** Supervises window lifecycle, enforces Windows UNC path sanitization, and cleans up child processes (`taskkill /F /T`).
   - **Node.js Orchestrator:** Acts as the **SOLE writer** to SQLite WAL storage. External sidecars and frontend clients have zero direct database access.
   - **FastAPI Python Sidecar:** Stateless compute worker restricted to local loopback (`127.0.0.1:8765`).

---

## 3. Reporting a Vulnerability

We deeply appreciate the efforts of security researchers and community members who help keep Omnitome secure.

### How to Report
Please report potential security vulnerabilities through **GitHub Private Vulnerability Reporting**:
👉 [Open a Private Security Advisory](https://github.com/ayumu-ml/Omnitome/security/advisories/new)

If you prefer direct email communication, you may contact the maintainers at:
- **Security Contact:** [ayumu.ml.contact@gmail.com](mailto:ayumu.ml.contact@gmail.com)
- **Subject Line:** `[Vulnerability Report] Omnitome - <Brief Vulnerability Title>`

### What to Include
To help us triage and resolve the issue quickly, please provide:
1. A clear description of the vulnerability and the affected subsystem (`src-tauri`, `backend`, `sidecar`, `frontend`).
2. Step-by-step instructions or a minimal proof-of-concept (PoC) script to reproduce the issue.
3. The potential impact (e.g., local privilege escalation, arbitrary code execution, denial of service).
4. Any suggested remediations or patches, if available.

---

## 4. Response Timeline & SLAs

We adhere to coordinated vulnerability disclosure:

| Milestone | Target Response Time |
| :--- | :--- |
| **Initial Acknowledgment** | Within **48 hours** |
| **Triage & Reproducibility Assessment** | Within **5 business days** |
| **Patch Release & Advisory Publication** | Within **14 business days** (for Critical/High severity) |

We will keep you informed of our progress throughout the remediation lifecycle and will credit your contribution in our security release notes (unless you prefer anonymity).

---

## 5. Safe Harbor

Any activities conducted in good faith within the scope of this policy will be considered authorized conduct:
- We will not initiate legal action against researchers for accidental, good-faith violations of this policy.
- Please avoid accessing, modifying, or destroying user data without authorization.
- Please give us reasonable time to remediate vulnerabilities before public disclosure.
