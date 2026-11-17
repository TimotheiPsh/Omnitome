/** Focused regressions; no model, network, or user database is accessed. */
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.resolve(__dirname, '../..');
function load(file, deps = {}, source) {
  const module = { exports: {} };
  vm.runInNewContext(source || fs.readFileSync(path.join(root, file), 'utf8'), {
    module, exports: module.exports,
    require: name => { if (!(name in deps)) throw Error(`Unexpected dependency: ${name}`); return deps[name]; },
    console: { log() {}, warn() {} }, setTimeout, clearTimeout,
  }, { filename: file });
  return module.exports;
}
const plain = value => JSON.parse(JSON.stringify(value));
const arbiterPath = 'backend/services/semantic/attributionArbiter.js';
function arbiter() { return load(arbiterPath, { '../ai/aiService': {} }); }
function params(score = .35) {
  return { paragraph: 'A claim requiring verification.', source_chunks: [{ id: 'a' }, { id: 'b' }],
    C_signal: { top1: score, top2: score - .01, margin: .01, top1ChunkId: 'a', weights: { a: .51, b: .49 } } };
}
for (const score of [.35, .55]) {
  for (const [name, outcome, reason] of [
    ['unavailable', () => { throw Error('offline'); }, 'arbiter_unavailable'],
    ['empty evidence', () => ({ chunk_ids: [], weights: {} }), 'arbiter_no_valid_sources'],
    ['invalid evidence', () => ({ chunk_ids: ['unknown'], weights: {} }), 'arbiter_no_valid_sources'],
  ]) {
    test(`arbitration ${name}, score ${score}: candidate IDs never become verified evidence`, async () => {
      const a = arbiter(); a.callSignalB = async () => outcome();
      const result = await a.resolveParagraphAttribution(params(score));
      assert.equal(result.grounded, false); assert.equal(result.method, 'unverified');
      assert.equal(result.confidence, 'none'); assert.equal(result.fallback_reason, reason);
      assert.deepEqual(plain(result.chunk_ids), ['a', 'b']);
    });
  }
}
test('fast bypass stays unverified; valid arbitration stays grounded', async () => {
  const a = arbiter(); a.callSignalB = async () => ({ chunk_ids: ['b'], weights: { b: 1 } });
  const skipped = await a.resolveParagraphAttribution({ ...params(), options: { fast: true } });
  assert.equal(skipped.grounded, false); assert.equal(skipped.fallback_reason, 'fast_path_bypass');
  const verified = await a.resolveParagraphAttribution(params());
  assert.equal(verified.grounded, true); assert.equal(verified.method, 'b_weak_c');
  assert.equal(verified.fell_back, false); assert.deepEqual(plain(verified.chunk_ids), ['b']);
});
test('existing strong, corroborated, and no-evidence decisions remain intact', async () => {
  const a = arbiter();
  const strong = params(.9); strong.C_signal.margin = .2;
  assert.equal((await a.resolveParagraphAttribution(strong)).method, 'c_primary');
  assert.equal((await a.resolveParagraphAttribution({ ...params(.55), A_claim: { weights: { a: 1 } } })).method, 'c_verified_by_a');
  assert.equal((await a.resolveParagraphAttribution(params(.1))).method, 'ungrounded');
});
function resolver(deps = {}) {
  const defaults = Object.fromEntries([
    './embeddingService', './attributionArbiter', '../../repositories/chapterRepository',
    '../../repositories/smartChapterRepository', '../../repositories/semanticChunkRepository',
    '../../repositories/outlineRepository', '../../repositories/jobRepository', '../../repositories/attributionRepository',
  ].map(k => [k, {}]));
  return load('backend/services/semantic/provenanceResolver.js', { ...defaults, ...deps });
}
for (const stage of ['legacy', 'reranker']) {
  test(`${stage}: unverified result persists without clickable segments and survives API formatting`, async () => {
    let rows = [];
    const a = arbiter(); a.callSignalB = async () => { throw Error('offline'); };
    const r = resolver({
      './attributionArbiter': a,
      './embeddingService': { getDimension: () => 2, embedText: async () => [1, 0], cosineSimilarity: () => .35 },
      '../../repositories/smartChapterRepository': { getById: () => ({ id: 'r', book_id: 'book',
        content: 'First claim. Second claim.', planned_source_section_ids: ['a', 'b'],
        metadata: { claimed_attributions: [{ paragraph_index: 0, weights_json: { attribution_stage: stage }, reranker_scores: { a: .9, b: .88 } }] } }) },
      '../../repositories/semanticChunkRepository': { getById: id => ({ id, embedding: [1, 0] }) },
      '../../repositories/attributionRepository': { getBySmartChapterId: () => rows, createBatch: x => (rows = x) },
      '../../repositories/jobRepository': { update() {} },
    });
    await r.verifyRepresentation('r', { skipJobCreation: true });
    assert.equal(rows[0].grounded, false); assert.equal(rows[0].segments.length, 0);
    assert.equal(rows[0].weights_json.sentence_splitter_version, 2);
    const api = r.getProvenance('r').paragraphs[0];
    assert.equal(api.method, 'unverified'); assert.equal(api.fell_back, true);
    assert.equal(api.fallback_reason, 'arbiter_unavailable');
  });
}
// Test 8's close-margin case: candidate proximity alone is not verification.
for (const fast of [false, true]) {
  test(`close-margin multi-chunk attribution with fast=${fast}`, async () => {
    let rows = [], calls = 0;
    const a = arbiter();
    a.callSignalB = async () => {
      calls += 1;
      return { chunk_ids: ['a', 'b'], weights: { a: .55, b: .45 } };
    };
    const r = resolver({
      './attributionArbiter': a,
      './embeddingService': { getDimension: () => 2, embedText: async () => [1, 0],
        cosineSimilarity: (_query, chunk) => chunk[0] },
      '../../repositories/smartChapterRepository': { getById: () => ({ id: 'r', book_id: 'book',
        content: 'Agile principles prioritize iterative delivery. Scrum uses timeboxed cycles.',
        planned_source_section_ids: ['a', 'b'], metadata: {} }) },
      '../../repositories/semanticChunkRepository': {
        getById: id => ({ id, embedding: [id === 'a' ? .85 : .80, 0] }) },
      '../../repositories/attributionRepository': { getBySmartChapterId: () => rows, createBatch: x => (rows = x) },
      '../../repositories/jobRepository': { update() {} },
    });
    await r.verifyRepresentation('r', { fast, force: true, skipJobCreation: true });
    assert.equal(calls, fast ? 0 : 1);
    assert.equal(rows[0].grounded, !fast);
    if (fast) {
      assert.equal(rows[0].fallback_reason, 'fast_path_bypass');
      assert.deepEqual(plain(rows[0].segments), []);
    } else {
      assert.equal(rows[0].method, 'b_arbitrated');
      assert.ok(rows[0].segments.length > 0);
      for (const segment of rows[0].segments) {
        assert.equal(segment.chunk_id, 'a');
        assert.deepEqual(plain(segment.chunk_ids), ['a', 'b']);
      }
    }
  });
}
test('historical failed rows normalize on read; successful rows are preserved', () => {
  const repo = load('backend/repositories/attributionRepository.js', { '../db/database': {} });
  const row = { id: 'old', grounded: 1, fell_back: 1, method: 'b_weak_c', confidence: 'low',
    fallback_reason: 'arbiter_unavailable', segments_json: '[{"sentence_start":0,"sentence_end":1,"chunk_id":"a"}]',
    source_chunk_ids: '["a"]', weights_json: '{"a":1}' };
  const normalized = repo.formatRow(row);
  assert.equal(normalized.grounded, false); assert.equal(normalized.method, 'unverified');
  assert.deepEqual(plain(normalized.segments), []); assert.deepEqual(plain(normalized.source_chunk_ids), ['a']);
  assert.equal(row.grounded, 1, 'read normalization does not mutate stored data');
  const valid = repo.formatRow({ ...row, fell_back: 0, fallback_reason: null });
  assert.equal(valid.grounded, true); assert.equal(valid.segments.length, 1);
});
// Compile the actual frontend function, rather than testing a hand-copied implementation.
const ts = require('../../frontend/node_modules/typescript');
const frontend = load('frontend/src/types/domain.ts', {}, ts.transpileModule(
  fs.readFileSync(path.join(root, 'frontend/src/types/domain.ts'), 'utf8'),
  { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } },
).outputText);
const backend = resolver();
const cases = [
  ['English', 'First sentence. Second sentence.', ['First sentence.', 'Second sentence.']],
  ['Korean', '첫 번째 문장입니다. 두 번째 문장입니다.', ['첫 번째 문장입니다.', '두 번째 문장입니다.']],
  ['Chinese', '第一句话。第二句话。', ['第一句话。', '第二句话。']],
  ['Japanese', '最初の文です。次の文です。', ['最初の文です。', '次の文です。']],
  ['fullwidth punctuation', '本当！？はい。', ['本当！？', 'はい。']],
  ['closing quotes', '「本当です。」次の文です。', ['「本当です。」', '次の文です。']],
  ['mixed language', 'First sentence. 다음 문장입니다. 最後です。', ['First sentence.', '다음 문장입니다.', '最後です。']],
  ['citations', '첫 문장입니다. [Source 1] 다음 문장입니다. [Source 2]', ['첫 문장입니다.', '다음 문장입니다.']],
  ['abbreviations', 'Dr. Smith uses Eq. 2. Results follow.', ['Dr. Smith uses Eq. 2.', 'Results follow.']],
  ['decimals', 'The value is 3.14. Next result.', ['The value is 3.14.', 'Next result.']],
  ['initials', 'J. R. Smith agrees. Next result.', ['J. R. Smith agrees.', 'Next result.']],
  ['math variable', 'Let the variable be x. Next result.', ['Let the variable be x.', 'Next result.']],
  ['CJK after abbreviation token', 'Use eq。下一句。', ['Use eq。', '下一句。']],
  ['question after abbreviation token', 'Ask Dr? Yes.', ['Ask Dr?', 'Yes.']],
  ['empty', '', []],
];
for (const [name, input, expected] of cases) {
  test(`sentence boundaries match in backend and frontend: ${name}`, () => {
    assert.deepEqual(plain(backend.splitIntoSentences(input)), expected);
    assert.deepEqual(plain(frontend.splitIntoSentences(input)), expected);
  });
}

for (const weight of [-1, 0, NaN, Infinity, 'high']) {
  test(`arbitration rejects invalid weight ${weight}`, async () => {
    const a = arbiter();
    a.callSignalB = async () => ({ chunk_ids: ['a'], weights: { a: weight } });
    const result = await a.resolveParagraphAttribution(params());
    assert.equal(result.grounded, false);
    assert.equal(result.fallback_reason, 'arbiter_invalid_weights');
  });
}
test('duplicate source IDs are deduplicated before weight normalization', async () => {
  const a = arbiter();
  a.callSignalB = async () => ({ chunk_ids: ['a', 'a', 'b'], weights: { a: 1, b: 1 } });
  const result = await a.resolveParagraphAttribution(params());
  assert.equal(result.grounded, true);
  assert.deepEqual(plain(result.chunk_ids), ['a', 'b']);
  assert.deepEqual(plain(result.weights), { a: .5, b: .5 });
});
test('real provider parsing accepts empty evidence without retrying', async () => {
  let calls = 0;
  const a = load(arbiterPath, { '../ai/aiService': {
    getActiveProvider: () => ({ checkHealth: async () => ({ available: true }) }),
    generateText: async () => { calls++; return { text: '{"chunk_ids":[],"weights":{}}' }; },
  } });
  const result = await a.resolveParagraphAttribution(params());
  assert.equal(result.fallback_reason, 'arbiter_no_valid_sources');
  assert.equal(calls, 1);
});
test('malformed provider JSON retries then remains unverified', async () => {
  let calls = 0;
  const a = load(arbiterPath, { '../ai/aiService': {
    getActiveProvider: () => ({ checkHealth: async () => ({ available: true }) }),
    generateText: async () => { calls++; return { text: 'not JSON' }; },
  } });
  const result = await a.resolveParagraphAttribution(params());
  assert.equal(result.fallback_reason, 'arbiter_unavailable');
  assert.equal(calls, 2);
});

for (const text of ['첫 문장입니다. 다음 문장입니다.', '第一句。第二句。', '「本当です。」次です。', '"First sentence." Next sentence.']) {
  test(`legacy segment indices are hidden when boundaries change: ${text}`, () => {
    const r = resolver();
    const row = { paragraph_index: 0, grounded: true, method: 'c_primary', weights: {},
      segments: [{ sentence_start: 0, sentence_end: 0, chunk_id: 'a' }] };
    const result = r.protectLegacySegments([row], { content: text })[0];
    assert.equal(result.grounded, false);
    assert.equal(result.fallback_reason, 'sentence_boundaries_changed');
    assert.equal(result.segments.length, 0);
    assert.equal(row.grounded, true);
    const fresh = { ...row, weights: { sentence_splitter_version: 2 } };
    assert.equal(r.protectLegacySegments([fresh], { content: text })[0], fresh);
  });
}
test('unchanged legacy English boundaries keep source links', () => {
  const r = resolver();
  const row = { paragraph_index: 0, grounded: true, weights: {}, segments: [{ sentence_start: 0, sentence_end: 1 }] };
  assert.equal(r.protectLegacySegments([row], { content: 'First sentence. Next sentence.' })[0], row);
});
test('API and cached verification both suppress stale source links', async () => {
  const row = { paragraph_index: 0, grounded: true, method: 'c_primary', weights: {},
    segments: [{ sentence_start: 0, sentence_end: 0, chunk_id: 'a' }] };
  const r = resolver({
    '../../repositories/attributionRepository': { getBySmartChapterId: () => [row] },
    '../../repositories/smartChapterRepository': { getById: () => ({ content: '첫 문장입니다. 다음 문장입니다.' }) },
  });
  const cached = await r.verifyRepresentation('r');
  const api = r.getProvenance('r');
  for (const response of [cached, api]) {
    assert.equal(response.paragraphs[0].grounded, false);
    assert.equal(response.paragraphs[0].fallback_reason, 'sentence_boundaries_changed');
    assert.equal(response.paragraphs[0].segments.length, 0);
  }
});
test('missing legacy content fails closed rather than guessing sentence indices', () => {
  const r = resolver();
  const row = { paragraph_index: 0, grounded: true, segments: [{ sentence_start: 0, sentence_end: 0 }] };
  assert.equal(r.protectLegacySegments([row], null)[0].grounded, false);
});
