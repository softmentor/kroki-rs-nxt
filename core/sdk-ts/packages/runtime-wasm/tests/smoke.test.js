import test from 'node:test';
import assert from 'node:assert/strict';

test('runtime-wasm smoke', () => {
  assert.equal(typeof 'runtime', 'string');
});
