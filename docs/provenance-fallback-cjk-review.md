# Provenance fallback and CJK sentence splitting review

## Behavior

Failed, skipped, or empty/invalid Signal B arbitration now produces `method: unverified`, `confidence: none`, and `grounded: false`. Candidate chunk IDs remain available for diagnostics, but no clickable sentence segments are generated. The reader distinguishes incomplete verification from a paragraph with no traced source. Existing fallback records are normalized when read; no database migration is needed. Successful existing C-only, C+A, and B decisions are unchanged.

Frontend and backend sentence splitting now support Korean sentence starts, Chinese/Japanese full-width terminal punctuation without intervening whitespace, and closing quotation marks. English abbreviation, initial, and decimal handling is retained.

## Verification

- `npm --prefix frontend ci --ignore-scripts --no-audit --no-fund`
- `npm run test:provenance`: 43 focused tests passed. These execute actual modules with mocked AI, embedding and repository boundaries, including persistence/API formatting and historical fallback normalization. The frontend splitter is transpiled from its actual TypeScript source and checked against the backend.
- `npm --prefix frontend run build`: TypeScript and production build passed; Vite reports a bundle size advisory.
- `git diff --check`: passed.

Tested on Node 24.22.1. The project recommends Node 22. The initial root dependency installation failed while downloading ONNX Runtime. On review follow-up, dependencies were installed with lifecycle scripts disabled and better-sqlite3 was rebuilt separately. Full database/Python integration validation remains incomplete (see below). Windows desktop, GPU inference, live provider failure, and visual interaction require follow-up validation before merge.

## Review limits and follow-up

This change prevents a verification failure from being presented as success; it does not establish that similarity-based successful decisions prove semantic entailment. Empty evidence and unavailable arbitration have distinct fallback reason codes, with conservative shared reader treatment.

Stored successful provenance segments are checked against the legacy splitter on read. If boundaries differ (including CJK and quoted English), their clickable segments are suppressed with `sentence_boundaries_changed` until force re-verification. Unaffected legacy paragraphs retain their links. New records persist `sentence_splitter_version: 2` inside the existing weights metadata, without a schema migration. Missing legacy content also fails closed. No automatic AI calls or database rewrites occur during reads. Force re-verification of affected chapters is required to restore their source links. Sentence splitting remains heuristic rather than a complete language parser.

Before merging, exercise a Windows chapter with a failing provider, successful retry using force re-verification, an older fallback record, and mixed Korean/Chinese/Japanese text. Verify source selection and text-to-speech highlighting against the original passages.

## Second review

- Deduplicate Signal B source IDs before normalizing weights. Reject negative, non-finite, nonnumeric, and all-zero weights rather than presenting them as verified evidence.
- Clear arbitration timeout handles after both success and failure; test actual provider parsing (mock provider responses) for empty evidence and malformed JSON retries.
- Restrict English abbreviation/initial exceptions to periods, so question marks and CJK punctuation cannot be swallowed by those rules.
- Protect old sentence indices in both the provenance API and cached verification responses. Tests cover old Korean, Chinese, Japanese, and quoted English, unchanged English, missing content, and new versioned records.

Remaining limitations: no semantic entailment benchmark, full database integration, Windows installer build, GPU test, or visual reader/TTS test has been performed. A model timeout clears the timer but does not cancel an in-progress provider request; provider-wide cancellation is separate work. The UI currently uses the shared verification-incomplete message for stale boundaries; a dedicated retry action would improve usability.


## Maintainer review follow-up: integration Test 8

- Test 8 now mocks a successful Signal B response for both source chunks and uses `fast: false`. It asserts that arbitration is called once, the paragraph is grounded through `b_arbitrated`, and both close-scoring chunks remain in the citation segments. The mock is restored in `finally`.
- Production verification behavior is unchanged. Two additional focused regressions cover the same close-margin case with successful arbitration and with fast-path bypass; the latter still returns no clickable segments.
- Validation: all 43 focused tests pass. The actual Test 8 block also passes in isolation with real SQLite repositories and a fresh temporary database, using the test's embedding and arbitration mocks. This is not a full-suite pass.
- Full `npm test` was attempted: the Python sidecar did not become ready, and execution waited in the embedding-dependent Build 3B suite. The run was stopped. A separate Build 5.1b run required the pre-existing `book-fixture-a` fixture for Test 6 and then waited at Test 7, before reaching Test 8; that run was also stopped. Please rerun the full suite in the configured Windows/Python environment before merging.
