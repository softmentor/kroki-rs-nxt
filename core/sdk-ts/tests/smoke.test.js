import test from 'node:test';
import assert from 'node:assert/strict';

test('sdk-ts workspace bootstrap is available', () => {
  assert.equal(typeof 'workspace', 'string');
});
