# Semantic Compression & Attribution Engine

At the heart of Omnitome is a strict thesis: **Loss-bounded semantic compression (~7:1 ratio), NOT summarization.**

Summarization discards nuances, flattens technical complexity, invents transitions, and answers the casual question *"What is the gist?"*. In contrast, semantic compression preserves every distinct concept, technical theorem, mathematical formula, character beat, and tactical revelation while densifying phrasing. Prompts and pipelines in Omnitome instruct models to *compress*, never to *summarize*.

Furthermore, Omnitome enforces **deterministic vector provenance**: every synthesized sentence must carry clickable backlinks directly to verbatim source chunks. Sentences lacking verified evidence are treated as defects.

---

## 1. Mathematical Grounding & Formal Invariants

Let a source document $\mathcal{D}_{\text{source}}$ consist of an ordered sequence of $N$ paragraphs $\mathcal{P} = \lbrace p_1, p_2, \dots, p_N \rbrace$. The derived Omni representation $\mathcal{D}_{\text{omni}}$ consists of $M$ synthesized chapters $\mathcal{S} = \lbrace S_1, S_2, \dots, S_M \rbrace$, where each chapter contains a sequence of sentences $S_k = (s_{k,1}, s_{k,2}, \dots, s_{k,|S_k|})$.

### 1.1 Semantic Compression Bound
The system enforces an explicit compression ratio:
$$C_r = \frac{\sum_{i=1}^N |W(p_i)|}{\sum_{k=1}^M |W(S_k)|} \approx 7.0 \pm 1.0$$
where $|W(\cdot)|$ evaluates effective word count (with CJK glyphs weighted at $\alpha = 0.70$).

Target chapter length is calibrated to **250–360 words** (with dynamic bounds between **240 and 500 words**). Any synthesis output falling below 150 words or exceeding 500 words triggers an automated prior-draft feedback retry loop.

### 1.2 Provenance Attribution Mapping
For every derived sentence $s \in S_k$, the provenance engine assigns an attribution set $\mathcal{A}(s) \subseteq \mathcal{P}$:
$$\mathcal{A}(s) = \lbrace p_i \in \mathcal{P} \mid \Phi(s, p_i) \ge \tau_{\text{provenance}} \rbrace$$
where the hybrid scoring function $\Phi(s, p_i)$ combines dense bi-encoder cosine similarity and FP16 cross-encoder re-ranking:
$$\Phi(s, p_i) = \lambda \cdot \cos\big(\mathbf{e}_{\text{dense}}(s), \mathbf{e}_{\text{dense}}(p_i)\big) + (1 - \lambda) \cdot \sigma\big(\mathrm{logit}_{\text{rerank}}(s, p_i)\big)$$

- If $\mathcal{A}(s) \neq \emptyset$: The sentence is decorated with interactive footnote markers (e.g., `[p12]`) that open the exact verbatim paragraph in the Original Research pane.
- If $\mathcal{A}(s) = \emptyset$: The sentence is flagged as an unattributed claim defect and styled in the UI with `text-ink-muted` and a subtle dotted underline.

---

## 2. The 5-Stage Ingestion Pipeline

Omnitome processes incoming documents through five sequential pipeline stages:

```
┌────────────────────────────────────────────────────────────────────────┐
│ STAGE 1: INGESTION & FAST PARSE                                        │
│ Extract AST, headings, paragraphs, and tables across 7 file formats   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ STAGE 2: CJK MULTILINGUAL TOKEN SAFETY & CHUNKING                      │
│ Script-aware word count (α = 0.70) · F51b hard chunk guards (<8,000c) │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ STAGE 3: SEMANTIC INDEXING & VECTOR EMBEDDING                          │
│ BGE-M3 (1024d) CLS pooling · FP16 CUDA batching · SQLite WAL commit    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ STAGE 4: EDITORIAL PLANNER & PROGRESSIVE SYNTHESIS                     │
│ Dynamic word budgets [250–360w] · Prior-draft retry loop               │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ STAGE 5: DETERMINISTIC PROVENANCE & ATTRIBUTION                        │
│ BGE Reranker v2 scoring · Bi-directional citation links · Defect style │
└────────────────────────────────────────────────────────────────────────┘
```

### Stage 1: Ingestion & Fast Parse
Extracts pristine source text from PDF, EPUB, DOCX, RTF, Markdown, Web URLs, and plain text. The parser preserves chapter boundaries, hierarchical headings, and mathematical formulas (rendered via KaTeX).

### Stage 2: CJK Multilingual Token Safety
CJK languages (Chinese, Japanese, Korean) do not separate words with whitespace. Splitting naively on whitespace (`text.split(/\s+/)`) reduces an entire 5,000-character Japanese chapter to a word count of 1, collapsing the editorial planner and causing massive memory blowouts (*痴人の愛* bug).
- **The Solution:** Omnitome uses script-aware word counting:
  ```javascript
  const latinWords = text.trim().split(/\s+/).filter(Boolean).length;
  const cjkChars = (text.match(/[\u3000-\u303F\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FFF\uAC00-\uD7AF]/g) || []).length;
  const totalWords = latinWords + Math.round(cjkChars * 0.7);
  ```
- **F51b Hard Chunk Guard:** Every chunk is guaranteed to be strictly under 8,000 characters before embedding, splitting at sentence boundaries if necessary.

### Stage 3: Semantic Indexing & Vector Embedding
The Python sidecar generates dense 1024-dimensional embeddings using **BGE-M3** with CLS pooling. Embeddings are batched in groups of 4 with a 2,048-token attention limit and committed to `storage/data.db` via Node's WAL queue.

### Stage 4: Editorial Planner & Progressive Synthesis
The planner analyzes the document structure and constructs a chapter budget map:
- **Contract Enforcement:** System prompts explicitly instruct the language model: *"Compress the following text preserving all causal steps, technical definitions, and tactical beats. Do NOT summarize or generalize."*
- **Prior-Draft Feedback Loop:** If an output violates the word count budget ([240, 500]), the prior draft is fed back into the prompt context with explicit instructions to tighten phrasing, preventing hallucinated additions on retries.
- **Empty Slice Bail-Out:** Slices falling below minimum character thresholds immediately bail out, preventing the model from hallucinating content from empty prompts.

### Stage 5: Deterministic Provenance & Attribution
Derived sentences are matched against source chunks using BGE Reranker v2 cross-encoders. Every sentence receives its provenance set $\mathcal{A}(s)$, generating interactive citations that allow readers to instantly cross-examine the compressed text against the authoritative original page.

---

## 3. Real-World Corpus Verification

The semantic engine is verified against a 10-fixture multi-format test gate (`scripts/run-corpus-tests.js`):

| Fixture Document | Format | Size | Chapters | Word Count | Throughput |
| :--- | :---: | ---:| ---:| ---:| ---:|
| *Moby Dick* | EPUB | 711.7 KB | 27 | 212,711 | 2.45M words/sec |
| *Pride and Prejudice* | EPUB | 548.0 KB | 16 | 128,191 | 2.36M words/sec |
| *Leaves of Grass* | EPUB | 675.9 KB | 290 | 7,515 | 186K words/sec |
| *Les Misérables (Tome 1)* | EPUB | 354.3 KB | 13 | 110,567 | 2.04M words/sec |
| Technical Sample | DOCX | 37.8 KB | 4 | 1,343 | 14K words/sec |
| Technical Sample | RTF | 7.7 KB | 1 | 1,343 | 84K words/sec |
| *Common Sense* | EPUB | 172.2 KB | 3 | 22,637 | 2.06M words/sec |
| *Chijin no Ai (痴人の愛)* | TXT | 218.5 KB | 15 | 76,765 | 1.68M words/sec |
| Research Transcript | MD | 266.5 KB | 9 | 36,906 | 449K words/sec |
| Prologue Sample | MD | 1.7 KB | 3 | 317 | 396K words/sec |
| **Release Gate Aggregate** | **All 5** | **2.92 MB** | **381** | **598,295** | **1.23M words/sec** |
