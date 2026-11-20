# Memory Invariants & VRAM Lifecycle

Omnitome is designed and benchmarked against a strict consumer hardware baseline: an **NVIDIA GeForce RTX 3050 Laptop GPU with 4,096 MiB VRAM** and **16 GB DDR4 system RAM**.

On an enterprise server with 80 GB H100 GPUs, loading multiple neural models simultaneously is trivial. On a 4 GB laptop GPU, running a 1024-dimensional embedding model, a cross-encoder reranker, and a neural speech model concurrently will instantly exhaust VRAM, trigger operating system page thrashing across the PCIe bus, or crash the process with PyTorch CUDA Out-Of-Memory (OOM) errors.

This document details the memory invariants and lifecycle governors that allow Omnitome to run enterprise-grade neural pipelines within a tight 4 GB VRAM / 296 MB RAM budget.

---

## 1. Hardware Baseline & Constraint Matrix

All production benchmarks and invariants are measured against this baseline:

| Subsystem | Hardware Specification | Constraints & Ceilings |
| :--- | :--- | :--- |
| **GPU Compute** | NVIDIA GeForce RTX 3050 Laptop GPU | 4,096 MiB VRAM total (~3.6 GB usable after OS display overhead) |
| **System Memory** | 16 GB DDR4-3200 (15.4 GB usable) | Must accommodate Windows OS, Tauri webview, and Node orchestrator |
| **CPU Architecture** | AMD Ryzen 7 4800H (8 Cores, 16 Threads) | Multithreaded parsing and audio playback |
| **Storage Subsystem** | NVMe PCIe M.2 SSD | SQLite WAL transactional writes |

---

## 2. Invariant 1: The F65 Sequential Await Barrier (Node.js RAM)

### The Hazard: The 14.6 GB Memory Explosion
In early versions of the ingestion pipeline, the Node.js orchestrator processed document chapters and paragraphs using unconstrained asynchronous promise maps (`Promise.all(chapters.map(...))`).

When ingesting a 508,000-word machine learning textbook or a 400-chapter web novel:
1. Thousands of concurrent async promises were spawned simultaneously.
2. In-flight text slices, intermediate JSON representations, and token buffers accumulated in the V8 heap before garbage collection could reclaim them.
3. Node.js process memory exploded to **14.6 GB RAM**, consuming 95% of available system memory on a 16 GB laptop and pushing Windows into severe disk swap.

### The Resolution: Sequential Await Barrier
Milestone F65 introduced the **Sequential Await Barrier**:
- Document ingestion, text slicing, and vector batching are forced through an explicit sequential await pipeline.
- Work is broken into bounded micro-batches: a chapter or slice must complete its database persistence before the next chunk buffer is allocated.
- **The Result:** Peak Node.js memory during heavy ingestion dropped from **14.6 GB down to 296 MB**—a **98% reduction in RAM footprint**—while actual wall-clock ingestion throughput accelerated by over 30x due to the elimination of OS paging.

```
+-------------------------------------------------------------------------------+
|                       INGESTION MEMORY FOOTPRINT                              |
+-------------------------------------------------------------------------------+
| Unconstrained Async:  [ 14.6 GB RAM - System Freezes & Paging ]              |
| F65 Await Barrier:    [ 296 MB RAM  - Bounded & Constant Heap ]  (-98%)       |
+-------------------------------------------------------------------------------+
```

---

## 3. Invariant 2: FP16 Vectorization & Attention Bounds (GPU VRAM)

### The Hazard: PCIe Page Thrashing
The primary embedding model in Omnitome is **BGE-M3**, which generates dense 1024-dimensional semantic representations and supports long input contexts. 

In standard FP32 precision, loading BGE-M3 consumed **3,959 MiB of VRAM**—virtually the entire capacity of the RTX 3050. When large document sections were embedded, GPU memory allocations exceeded physical VRAM. Windows transparently paged tensor memory into system RAM across PCIe, resulting in catastrophic latency: embedding a 16-chunk batch required **~124 seconds**.

### The Resolution: FP16 Execution & Sequence Bounds
1. **PyTorch FP16 CUDA:** Model weights and attention computations were converted to half-precision (`torch.float16`). Active VRAM dropped from **3,959 MiB to 1,135 MB** (a 71% reduction).
2. **Bounded Sequence Length:** Maximum attention sequence length was capped at 2,048 tokens, preventing quadratic attention memory blowouts on giant chapters.
3. **Micro-Batching:** Embeddings are dispatched in micro-batches of 4.
4. **The Result:** Embedding the same 16-chunk batch dropped from **124 seconds to under 1 second**—a **120x+ speedup**.

---

## 4. Invariant 3: F43 LifecycleManager (Dynamic Idle VRAM Eviction)

### The Multi-Model Contention Problem
Omnitome features four separate GPU/neural workloads:
- **BGE-M3 Embedder:** ~1.1 GB VRAM (FP16)
- **BGE Reranker v2:** ~1.8 GB VRAM (FP16)
- **Kokoro-82M TTS:** ~600 MB VRAM
- **PaddleOCR:** ~800 MB VRAM

Total concurrent footprint: `1.1 + 1.8 + 0.6 + 0.8 = ~4.3 GB VRAM`. On a 4.0 GB card, running these models concurrently guarantees a CUDA Out-Of-Memory exception.

### The Lifecycle State Machine
Milestone F43 implemented the **LifecycleManager** inside the Python sidecar. Models transition through five deterministic states:

```
UNLOADED ──(Request)──> LOADING ──> READY ──(Inactivity)──> COOLING_DOWN ──(TTL Expired)──> EVICTED
   ▲                                                                                              │
   └───────────────────────────(torch.cuda.empty_cache())────────────────────────────────────────┘
```

### Sliding Idle TTL Eviction Timers
Each model is assigned an independent sliding Time-To-Live (TTL) counter that resets upon each inference request:
- **PaddleOCR:** 300 seconds (5 minutes)
- **Kokoro-82M TTS:** 300 seconds (5 minutes)
- **BGE Reranker v2:** 600 seconds (10 minutes)
- **BGE-M3 Embedder:** 900 seconds (15 minutes)

### Safe Cache Reclaiming
When a model's TTL expires:
1. Model references are dereferenced from the global registry.
2. The Python garbage collector is invoked: `gc.collect()`.
3. PyTorch releases reserved memory back to the Windows graphics subsystem: `torch.cuda.empty_cache()`.
4. **VRAM Reclaimed:** **3.2 GB of VRAM** is returned to the operating system, dropping the Python sidecar's idle VRAM consumption to just **174 MB**.

### Transparent On-Demand Reloading
When the user resumes reading or requests speech narration after an eviction:
- Kokoro TTS rewarms within **~1.5–2.0 seconds**, indicated by a subtle status chip in the UI.
- BGE-M3 reloads from NVMe in **~6.5 seconds**.
- Audio decks utilize `preload="none"` to ensure playback elements do not hold memory buffers indefinitely.

---

## 5. Invariant 4: Input-Calibrated Reranker Timeouts

Standard fixed API timeouts (e.g., 10 seconds) fail on dense literature. When cross-encoding a 200-chapter document, ranking candidate source passages against synthesized claims requires dynamic computational time.

Under milestone F71, Omnitome replaced fixed timeouts with **Dynamic Reranker Timeouts**:
$$\text{Timeout}_{\text{rerank}} = \operatorname{clamp}\Big(8.0\text{s},\; \frac{|\mathcal{C}|}{k_{\text{factor}}} \times 1.25,\; 60.0\text{s}\Big)$$
where $|\mathcal{C}|$ represents candidate passage count. This guarantees zero false timeout fallbacks during heavy indexing while protecting the UI from indefinite hangs.
