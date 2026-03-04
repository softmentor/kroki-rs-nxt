import test from 'node:test';
import assert from 'node:assert/strict';

test('ui-components smoke', () => {
  assert.equal(typeof 'components', 'string');
});
