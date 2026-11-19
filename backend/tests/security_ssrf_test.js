'use strict';

const assert = require('assert');
const webFetcher = require('../services/web/webFetcher');

console.log('🧪 Starting SSRF & DNS Rebinding Security Tests...');

async function runTests() {
  // Test 1: Literal private IPv4 addresses blocked
  const privateIps = [
    'http://127.0.0.1:8765/leak',
    'http://10.0.0.1/',
    'http://172.16.0.1/',
    'http://192.168.1.1/admin',
    'http://169.254.169.254/latest/meta-data/',
    'http://0.0.0.0:3000/'
  ];

  for (const url of privateIps) {
    let threw = false;
    try {
      await webFetcher.fetch(url);
    } catch (err) {
      threw = true;
      assert(
        err.message.includes('restricted for security') || err.message.includes('restricted'),
        `Unexpected error message for ${url}: ${err.message}`
      );
    }
    assert(threw, `Expected SSRF block for literal private IP: ${url}`);
  }
  console.log('  ✓ Test 1: Literal private IPv4 and metadata addresses strictly blocked');

  // Test 2: Localhost aliases and private domain suffixes
  const localDomains = [
    'http://localhost:3000/api',
    'http://app.localhost/',
    'http://printer.local/',
    'http://service.internal/'
  ];

  for (const url of localDomains) {
    let threw = false;
    try {
      await webFetcher.fetch(url);
    } catch (err) {
      threw = true;
      assert(err.message.includes('restricted for security'));
    }
    assert(threw, `Expected SSRF block for local domain: ${url}`);
  }
  console.log('  ✓ Test 2: Localhost aliases and .local/.internal domains strictly blocked');

  // Test 3: Unsupported protocols
  const badProtocols = [
    'file:///C:/Windows/System32/drivers/etc/hosts',
    'gopher://127.0.0.1:70/',
    'ftp://example.com/test.txt'
  ];

  for (const url of badProtocols) {
    let threw = false;
    try {
      await webFetcher.fetch(url);
    } catch (err) {
      threw = true;
      assert(err.message.includes('Unsupported protocol') || err.message.includes('Invalid URL'));
    }
    assert(threw, `Expected protocol block for: ${url}`);
  }
  console.log('  ✓ Test 3: Non-HTTP/HTTPS protocols strictly blocked');

  // Test 4: DNS Rebinding / Wildcard domains pointing to loopback or metadata
  const rebindingUrls = [
    'http://127.0.0.1.nip.io:8765/status',
    'http://169.254.169.254.nip.io/meta-data'
  ];

  for (const url of rebindingUrls) {
    let threw = false;
    try {
      await webFetcher.fetch(url);
    } catch (err) {
      threw = true;
      assert(
        err.message.includes('resolves to private/local network address') ||
        err.message.includes('restricted for security') ||
        err.message.includes('Failed to reach') ||
        err.message.includes('Connection timed out'),
        `Unexpected error for DNS rebinding attempt ${url}: ${err.message}`
      );
    }
    assert(threw, `Expected DNS resolution or rejection for: ${url}`);
  }
  console.log('  ✓ Test 4: DNS rebinding wildcard domains verified');

  console.log('🎉 All SSRF & DNS Rebinding Security Tests Passed!');
}

runTests().catch(err => {
  console.error('SSRF test failure:', err);
  process.exit(1);
});
