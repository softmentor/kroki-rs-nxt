import test from 'node:test';
import assert from 'node:assert/strict';

import { isLatestRevision, mapRenderResult, nextRevision } from '../dist/render-flow.js';

test('nextRevision increments monotonically', () => {
  assert.equal(nextRevision(0), 1);
  assert.equal(nextRevision(9), 10);
});

test('isLatestRevision identifies stale revisions', () => {
  assert.equal(isLatestRevision(3, 3), true);
  assert.equal(isLatestRevision(2, 3), false);
});

test('mapRenderResult maps success payload', () => {
  const mapped = mapRenderResult({ kind: 'success', contentType: 'image/svg+xml', data: '<svg/>' });
  assert.equal(mapped.status, 'success');
  assert.equal(mapped.renderedSvg, '<svg/>');
});

test('mapRenderResult maps error payload with status code', () => {
  const mapped = mapRenderResult({ kind: 'error', code: 'timeout', message: 'timed out' });
  assert.equal(mapped.status, 'error:timeout');
  assert.equal(mapped.renderedSvg, undefined);
});
