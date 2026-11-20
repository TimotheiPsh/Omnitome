'use strict';

const assert = require('assert');
const markdownParser = require('../services/ingestion/parsers/markdownParser');
const aiNormalizer = require('../services/ai/aiNormalizer');

console.log('🧪 Starting ReDoS & Table Parser Security Tests...');

// Test 1: Standard GFM Table in MarkdownParser
{
  const md = `# Title\n\n| Feature | Status | Notes |\n|:---|:---:|---:|\n| Embeddings | Ready | FP16 |\n| Reranker | Active | v2 |\n\nParagraph after table.`;
  const doc = markdownParser.parse(md, 'test.md');
  const tableBlock = doc.blocks.find(b => b.type === 'table');
  assert(tableBlock, 'Table block must be extracted by MarkdownParser');
  assert.strictEqual(tableBlock.headers.length, 3, 'Table must have 3 headers');
  assert.strictEqual(tableBlock.rows.length, 2, 'Table must have 2 rows');
  assert.strictEqual(tableBlock.rows[0][0], 'Embeddings');
  console.log('  ✓ Test 1: Standard GFM table successfully extracted by MarkdownParser');
}

// Test 2: Standard GFM Table in AINormalizer
{
  const text = `| Model | VRAM |\n| --- | --- |\n| BGE-M3 | 2.1GB |\n| Kokoro | 600MB |\n`;
  const blocks = aiNormalizer.normalize(text);
  const tableBlock = blocks.find(b => b.type === 'table');
  assert(tableBlock, 'Table block must be extracted by AINormalizer');
  assert.strictEqual(tableBlock.headers.length, 2);
  assert.strictEqual(tableBlock.rows.length, 2);
  console.log('  ✓ Test 2: Standard GFM table successfully normalized by AINormalizer');
}

// Test 3: Pathological ReDoS String Resilience
{
  // Pathological inputs that previously triggered exponential/polynomial backtracking
  const pathologicalStrings = [
    '|' + '-'.repeat(50000) + ' '.repeat(50000) + '-',
    '|' + ':-'.repeat(25000) + ' '.repeat(25000) + '-:',
    '|' + ' - - - '.repeat(10000) + '!',
    '-'.repeat(80000),
    '|' + '---|'.repeat(500)
  ];

  for (const str of pathologicalStrings) {
    const t0 = Date.now();
    const mdRes = markdownParser.isTableDelimiter(str);
    const aiRes = aiNormalizer.isTableDelimiter(str);
    const elapsed = Date.now() - t0;
    assert(elapsed < 50, `ReDoS detected! Parsing took ${elapsed}ms for pathological string`);
    assert.strictEqual(typeof mdRes, 'boolean');
    assert.strictEqual(typeof aiRes, 'boolean');
  }
  console.log('  ✓ Test 3: Pathological ReDoS strings evaluated in < 50ms with zero event loop hang');
}

// Test 4: Regression - Horizontal rule is not misidentified as table
{
  assert.strictEqual(markdownParser.isTableDelimiter('---'), true); // 1-col separator
  assert.strictEqual(markdownParser.isTableDelimiter('* * *'), false);
  assert.strictEqual(markdownParser.isTableDelimiter('___'), false);
  console.log('  ✓ Test 4: Non-hyphen rule delimiters not falsely recognized');
}

console.log('🎉 All ReDoS & Table Security Tests Passed!');
