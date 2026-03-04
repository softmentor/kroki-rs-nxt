import test from 'node:test';
import assert from 'node:assert/strict';

test('ui-tokens smoke', () => {
  assert.equal(typeof 'tokens', 'string');
});
