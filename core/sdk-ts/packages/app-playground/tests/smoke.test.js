import test from 'node:test';
import assert from 'node:assert/strict';

test('app-playground smoke', () => {
  assert.equal(typeof 'playground', 'string');
});
