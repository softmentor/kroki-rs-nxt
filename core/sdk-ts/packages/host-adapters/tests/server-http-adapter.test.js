import test from 'node:test';
import assert from 'node:assert/strict';

import { RuntimeHostAdapter, ServerHttpAdapter } from '../dist/index.js';

test('RuntimeHostAdapter forwards render to bridge', async () => {
  let called = false;
  const bridge = {
    async render(request, signal) {
      called = true;
      assert.equal(request.diagramType, 'd2');
      assert.equal(Boolean(signal), true);
      return { kind: 'success', contentType: 'image/svg+xml', data: '<svg/>' };
    },
  };
  const adapter = new RuntimeHostAdapter(bridge);
  const controller = new AbortController();

  const result = await adapter.render(
    { diagramType: 'd2', outputFormat: 'svg', source: 'a -> b', revisionId: 1 },
    controller.signal,
  );

  assert.equal(called, true);
  assert.equal(result.kind, 'success');
});

test('ServerHttpAdapter maps HTTP success response', async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => ({
    ok: true,
    status: 200,
    async json() {
      return { data: '<svg></svg>' };
    },
  });

  try {
    const adapter = new ServerHttpAdapter('/render');
    const result = await adapter.render({ diagramType: 'd2', outputFormat: 'svg', source: 'a -> b', revisionId: 1 });

    assert.equal(result.kind, 'success');
    assert.equal(result.contentType, 'image/svg+xml');
    assert.equal(result.data, '<svg></svg>');
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('ServerHttpAdapter maps error body to RenderFailure', async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = async () => ({
    ok: false,
    status: 422,
    async json() {
      return { error: { code: 'validation_error', message: 'Invalid source' } };
    },
  });

  try {
    const adapter = new ServerHttpAdapter('/render');
    const result = await adapter.render({ diagramType: 'graphviz', outputFormat: 'svg', source: '', revisionId: 2 });

    assert.equal(result.kind, 'error');
    assert.equal(result.code, 'validation_error');
    assert.equal(result.message, 'Invalid source');
  } finally {
    globalThis.fetch = originalFetch;
  }
});
