import test from 'node:test';
import assert from 'node:assert/strict';

import { RuntimeBridgeStub } from '../dist/index.js';

test('RuntimeBridgeStub render returns success payload with svg content type', async () => {
  const bridge = new RuntimeBridgeStub();
  const result = await bridge.render({
    diagramType: 'd2',
    outputFormat: 'svg',
    source: 'a -> b',
    revisionId: 7,
  });

  assert.equal(result.kind, 'success');
  assert.equal(result.contentType, 'image/svg+xml');
  assert.match(result.data, /Stub render for d2/);
  assert.match(result.data, /Revision 7/);
});

test('RuntimeBridgeStub getCapabilities includes expected providers', async () => {
  const bridge = new RuntimeBridgeStub();
  const capabilities = await bridge.getCapabilities();

  assert.ok(capabilities.includes('graphviz'));
  assert.ok(capabilities.includes('mermaid'));
  assert.ok(capabilities.includes('vegalite'));
});
