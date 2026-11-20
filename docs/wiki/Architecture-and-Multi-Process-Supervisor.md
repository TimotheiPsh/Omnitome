# Architecture & Multi-Process Supervisor

Omnitome is engineered as a coordinated multi-process desktop system. Rather than attempting to run machine learning models, database persistence, and UI rendering within a single monolithic runtime, Omnitome isolates concerns across specialized process boundaries:

```
┌─────────────────────────────────────────────────────────────────┐
│                 Tauri v2 Desktop Shell (Rust)                   │
│             Window Lifecycle & Child-Process Supervisor         │
└────────────────┬───────────────────────────────┬────────────────┘
                 │ Local Webview IPC             │ Spawns & Supervises
                 ▼                               ▼
 ┌──────────────────────────────┐ ┌─────────────────────────────┐
 │ Node.js 22 LTS Orchestrator  │ │ FastAPI Python Sidecar      │
 │ • SOLE SQLite WAL writer     │ │ • BGE-M3 (FP16 CUDA)        │
 │ • Sequential Await Barrier   │ │ • BGE Reranker v2           │
 │ • Ingestion Queue Daemon     │ │ • Kokoro-82M Neural TTS     │
 │ • Peak RAM: ~296 MB          │ │ • Idle VRAM Eviction (3.2GB)│
 └──────────────┬───────────────┘ └──────────────┬──────────────┘
                │ HTTP REST (Port 3000)          │ HTTP REST (Port 8765)
                ▼                                │
 ┌──────────────────────────────┐                │
 │ SQLite Database (data.db)    │◄───────────────┘ (REST via Node ONLY)
 │ Local WAL Mode               │
 └──────────────────────────────┘
```

---

## 1. Process Roles & Boundaries

### 1.1 Tauri v2 Desktop Shell (Rust)
The Tauri container is written in Rust (`src-tauri/`) and provides the native operating system boundary:
- **Child-Process Supervisor:** Spawns and monitors both the Node.js orchestrator and the Python sidecar.
- **Dynamic Port Handshake:** Discovers available ephemeral TCP ports at runtime to prevent port collisions with other applications.
- **First-Run Extraction:** Unpacks the LZMA-compressed sidecar archive (`resources/sidecar.7z`) into `%LOCALAPPDATA%\Omnitome\sidecar\` on initial launch.
- **Clean Process Tree Teardown:** Intercepts window close and application termination events. On Windows, child processes spawned by intermediate shells can easily become orphaned. The supervisor issues a tree kill command (`taskkill /F /T /PID <pid>`) ensuring zero zombie Python or Node processes remain in memory when the user exits.

### 1.2 Node.js 22 LTS Orchestrator
The Node.js service (`backend/`) acts as the central coordinator and sole persistence layer:
- **The Sole SQLite Writer Invariant:** Node.js manages `better-sqlite3` in Write-Ahead Logging (WAL) mode (`storage/data.db`). **Neither the Python sidecar, Tauri shell, nor Frontend clients may ever open or write to `data.db` directly.** All database reads and writes originate from Node repositories and services.
- **Sequential Ingestion Barrier:** Controls the 5-stage ingestion pipeline behind an await barrier, eliminating promise swarms and holding peak Node memory to 296 MB.
- **Background Queue Daemon:** Continuously monitors the task queue in WAL mode, processing document parsing, chunking, classification, and vector indexing asynchronously.
- **Provider Routing:** Marshals calls between external LLM backends (Ollama, LM Studio, OpenAI-compatible local servers) and local vector operations.

### 1.3 FastAPI / Python 3.10+ Sidecar
The Python microservice (`sidecars/python/`) runs at `127.0.0.1:8765` and owns all compute-heavy ML and NLP workloads:
- **PyTorch FP16 CUDA Acceleration:** Executes dense multilingual embeddings with BGE-M3 (1024-dimensional vectors) and cross-encoder reranking with BGE Reranker v2.
- **Kokoro-82M Voice Synthesis:** Performs high-fidelity offline neural speech generation across curated English voices (`af_heart`, `am_adam`, `bf_alice`, `bm_george`).
- **OCR & Document Extraction:** Runs PaddleOCR for degraded scanned documents and PyMuPDF (`fitz`) for PDF structural AST extraction.
- **Stateless Operation:** The Python sidecar stores no durable state in SQLite. It processes inputs and returns payloads strictly over REST endpoints to Node.js.

### 1.4 Frontend Client (React 18.3, Vite, Tailwind CSS)
The frontend (`frontend/`) renders inside the Tauri WebView2 container:
- Communicates **exclusively with the Node.js orchestrator** via HTTP REST and Server-Sent Events (SSE).
- Provides single-pane and dual-pane views (Original Research vs. Omni Derived Reading).
- Synchronizes audio narration decks with millisecond sentence highlighting.
- Typesets LaTeX mathematical formulas with KaTeX.

---

## 2. Invariant Governance & Concurrency Rules

### The Sole SQLite Writer Rule
Multi-process access to SQLite files often leads to `SQLITE_BUSY` database locking or silent database corruption. Omnitome eliminates this risk through architectural separation:
1. Only the Node.js orchestrator initializes a `better-sqlite3` connection.
2. WAL mode (`PRAGMA journal_mode = WAL;`) allows concurrent read transactions while writes are serialized inside Node's single-threaded event loop.
3. When Python needs document chunks or chapter metadata, it receives the data via Node's HTTP payload. When Python finishes vector embedding or reranking, it returns the floating-point arrays to Node, which commits them to `storage/data.db`.

### Dynamic Port Negotiation
Hardcoding ports (e.g., `3000` and `8765`) invites launch crashes if another local development server or background daemon occupies those ports:
1. Tauri probes the OS for free ephemeral ports.
2. The chosen ports are injected into the child process environment variables (`PORT` and `PYTHON_SIDECAR_PORT`).
3. The frontend receives the configured endpoints through initialization configuration before initiating API calls.

### Clean Shutdown Protocol
When the user closes the Omnitome desktop window:
1. Tauri catches the `WindowEvent::CloseRequested` event.
2. A graceful shutdown signal (`SIGTERM` / `HTTP /shutdown`) is dispatched to Node.js and FastAPI to flush in-flight WAL buffers and release model locks.
3. If processes do not terminate within a 1,500ms safety window, the Rust supervisor executes:
   ```cmd
   taskkill /F /T /PID <node_pid>
   taskkill /F /T /PID <python_pid>
   ```
4. This guarantees that neither background GPU threads nor Node worker threads leak into the host OS.
