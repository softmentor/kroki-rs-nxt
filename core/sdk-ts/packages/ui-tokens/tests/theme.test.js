import test from 'node:test';
import assert from 'node:assert/strict';

import { applyTheme } from '../dist/index.js';

test('applyTheme sets data-theme attribute', () => {
  const attrs = new Map();
  const originalDocument = globalThis.document;

  globalThis.document = {
    documentElement: {
      setAttribute(name, value) {
        attrs.set(name, value);
      },
    },
  };

  try {
    applyTheme('dark');
    applyTheme('light');
    assert.equal(attrs.get('data-theme'), 'light');
  } finally {
    globalThis.document = originalDocument;
  }
});
