# Omnitome Documentation Wiki

Welcome to the technical documentation and engineering wiki for **Omnitome** (formerly *Smart-Reader*), a local-first neural reading system featuring loss-bounded semantic compression, deterministic vector provenance, and on-device Kokoro-82M speech synthesis.

Omnitome runs entirely on consumer hardware with zero cloud dependencies or network egress, preserving the immutable ground truth of source literature while generating verifiable reading lenses.

---

## Foundational Invariants: The Three Laws

Omnitome's architecture is anchored by three strict product and engineering laws:

```
+-------------------------------------------------------------------------------+
|                       THE THREE FOUNDATIONAL LAWS                             |
+-------------------------------------------------------------------------------+
| 1. Law 1: Source is paper.                                                    |
|    Original documents (PDF, EPUB, DOCX, RTF, Markdown, TXT, Web) are immutable|
|    ground truth. Verbatim text is never rewritten, deleted, or truncated.     |
|                                                                               |
| 2. Law 2: The lens is tinted.                                                 |
|    Derived Omni representations are explicitly calibrated (~250-360 words per |
|    chapter) and styled with theme-aware typographic tokens.                   |
|                                                                               |
| 3. Law 3: Derivation never masquerades as source.                             |
|    Every synthesized claim maintains 100% deterministic, clickable citations  |
|    back to verbatim source chunks. Sentences lacking provenance are flagged   |
|    as structural defects.                                                     |
+-------------------------------------------------------------------------------+
```

---

## Technical Deep Dives

Explore the core architectural subsystems of Omnitome:

| Guide | Core Topics |
| :--- | :--- |
| **[Architecture & Multi-Process Supervisor](Architecture-and-Multi-Process-Supervisor)** | Tauri v2 (Rust) process supervision, Node.js 22 LTS orchestrator (Sole SQLite Writer), FastAPI/PyTorch sidecar, ephemeral port handshakes, and clean process tree teardown (`taskkill /F /T`). |
| **[Memory Invariants & VRAM Lifecycle](Memory-Invariants-and-VRAM-Lifecycle)** | Operating on 4 GB VRAM (RTX 3050 Laptop baseline), the F65 Sequential Await Barrier (dropping Node RAM from 14.6 GB to 296 MB), and F43 LifecycleManager (reclaiming 3.2 GB idle VRAM). |
| **[Semantic Compression & Attribution Engine](Semantic-Compression-and-Attribution-Engine)** | The "compression, not summarization" contract, BGE-M3 (1024d) CLS pooling, cross-encoder reranking, CJK multilingual token safety, and deterministic sentence attribution. |
| **[Packaging & Windows Distribution](Packaging-and-Windows-Distribution)** | Bypassing the 32-bit NSIS 2 GB installer ceiling via `sidecar.7z` LZMA extraction, PyInstaller cuDNN DLL harvesting, WebView2 audio autoplay unlock, and Windows verbatim UNC path sanitization. |

---

## Consumer Hardware Baseline

Every performance metric, memory boundary, and algorithmic optimization in Omnitome is benchmarked and verified on a standard consumer laptop:

- **Host Machine:** Windows 11 Home x64
- **CPU:** AMD Ryzen 7 4800H (8 Cores, 16 Threads @ 2.9–4.2 GHz)
- **RAM:** 16 GB DDR4 (15.4 GB usable)
- **GPU:** NVIDIA GeForce RTX 3050 Laptop GPU (4,096 MiB VRAM, PyTorch FP16 CUDA)
- **Runtimes:** Node.js v22 LTS · Python 3.10+ FastAPI · Tauri v2 (Rust 1.80+)

---

## Verification & Test Suites

Omnitome enforces rigorous gate assertions across the entire multi-process stack:

- **Node.js Orchestrator:** 61 unit and regression suites covering queue lifecycles, SQLite WAL concurrency, word count algorithms, and fallback handlers.
- **Tauri Supervisor:** 17 Rust unit tests validating port allocation, process spawning, and verbatim path stripping.
- **Release Gate Corpus:** 10 multi-format fixtures (EPUB, PDF, DOCX, RTF, TXT, MD) across narrative, poetry, multilingual (Aozora, French), and pathological edge-case inputs.
- **Procedural Sound Engine:** 90 offline DSP PCM WAV stems verified for -3 dBFS peak normalization and sub-700ms build velocity.
