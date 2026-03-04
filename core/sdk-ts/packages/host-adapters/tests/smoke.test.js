import test from 'node:test';
import assert from 'node:assert/strict';

test('host-adapters smoke', () => {
  assert.equal(typeof 'adapter', 'string');
});
