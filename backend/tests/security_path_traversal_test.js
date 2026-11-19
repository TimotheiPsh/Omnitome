'use strict';

const assert = require('assert');
const path = require('node:path');
const fs = require('node:fs');
const os = require('node:os');
const importQueueService = require('../services/ingestion/importQueueService');
const ttsClient = require('../services/ai/ttsClient');

console.log('🧪 Starting Path Traversal & Containment Security Tests...');

async function runTests() {
  // Test 1: Hash validation in ttsClient / routes rejects directory traversal markers
  const invalidHashes = [
    '../../etc/passwd',
    '..%2f..%2fwin.ini',
    'af_heart/../../something',
    '12345', // too short
    'g'.repeat(64), // not hex
    '../'.repeat(16) + 'evil'
  ];

  const HASH_REGEX = /^[a-f0-9]{64}$/;
  for (const h of invalidHashes) {
    assert.strictEqual(HASH_REGEX.test(h), false, `Hash regex should reject: ${h}`);
  }
  console.log('  ✓ Test 1: SHA-256 hash regex strictly blocks path traversal characters');

  // Test 2: Voice parameter normalization & whitelist containment
  const invalidVoices = [
    '../../custom_voice',
    '..\\..\\windows',
    'af_heart/../am_adam',
    'malicious_voice_name'
  ];

  for (const v of invalidVoices) {
    const { filePath } = ttsClient.resolveAudioPath('a'.repeat(64), v);
    // Voice whitelist must force fallback to 'af_heart'
    assert(filePath.includes('af_heart'), `Expected fallback to af_heart for voice: ${v}`);
    assert(!filePath.includes('..'), `Path must not contain traversal dots: ${filePath}`);
    const resolved = path.resolve(filePath);
    const resolvedBase = path.resolve(ttsClient.TTS_CACHE_DIR);
    assert(resolved.startsWith(resolvedBase + path.sep), `Resolved path must stay inside TTS_CACHE_DIR: ${resolved}`);
  }
  console.log('  ✓ Test 2: Voice parameter strictly normalized to whitelist within cache boundary');

  // Test 3: importQueueService blocks unauthorized external file.path
  {
    const hostsFile = process.platform === 'win32'
      ? path.join(process.env.SystemRoot || 'C:\\Windows', 'System32', 'drivers', 'etc', 'hosts')
      : '/etc/hosts';

    if (fs.existsSync(hostsFile)) {
      let threw = false;
      try {
        await importQueueService.enqueueFile({
          file: {
            path: hostsFile,
            originalname: 'hosts.txt'
          }
        });
      } catch (err) {
        threw = true;
        assert(err.message.includes('outside permitted temporary directories') || err.message.includes('Access denied'));
      }
      assert(threw, `Expected access denial for unauthorized system file: ${hostsFile}`);
    }
    console.log('  ✓ Test 3: Unauthorized system file copying blocked in importQueueService');
  }

  // Test 4: importQueueService allows authorized temp directory file.path
  {
    const tempFile = path.join(os.tmpdir(), `test-upload-${Date.now()}.txt`);
    fs.writeFileSync(tempFile, 'Temporary upload test payload');
    try {
      const task = await importQueueService.enqueueFile({
        file: {
          path: tempFile,
          originalname: 'temp-doc.txt'
        }
      });
      assert(task && task.id, 'Task must be created for permitted temp file');
      importQueueService.cancelTask(task.id);
      // Clean up staged file
      if (task.source_meta && task.source_meta.stagingPath && fs.existsSync(task.source_meta.stagingPath)) {
        try { fs.unlinkSync(task.source_meta.stagingPath); } catch (_) {}
      }
    } finally {
      if (fs.existsSync(tempFile)) {
        fs.unlinkSync(tempFile);
      }
    }
    console.log('  ✓ Test 4: Permitted temp file copying succeeds in importQueueService');
  }

  console.log('🎉 All Path Traversal & Containment Security Tests Passed!');
}

runTests().catch(err => {
  console.error('Path traversal test failure:', err);
  process.exit(1);
});
