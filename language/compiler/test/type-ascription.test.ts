import { describe, test } from 'node:test';
import assert from 'node:assert';
import { compileFromString } from '../src/api.js';

describe('Type Ascription', () => {
  test('parses type ascription syntax', () => {
    const code = `λf()→ℤ=(42:ℤ)\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('typechecks correct ascription', () => {
    const code = `λf()→𝕊=("hello":𝕊)\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('rejects incorrect ascription', () => {
    const code = `λf()→ℤ=("hello":ℤ)\n`;  // String ascribed as Int
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, false);
    if (!result.ok) {
      assert.match(result.error.message, /type mismatch|Literal type mismatch/i);
    }
  });

  test('allows empty list with ascription', () => {
    const code = `λf()→[ℤ]=([]:[ℤ])\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('rejects let without ascription', () => {
    const code = `λf()→ℤ=l x=42;x\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, false);
    if (!result.ok) {
      assert.strictEqual(result.error.code, 'SIGIL-CANON-LET-UNTYPED');
    }
  });

  test('accepts let with ascription', () => {
    const code = `λf()→ℤ=l x=(42:ℤ);x+1\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('nested let bindings with ascription', () => {
    const code = `λf()→ℤ=l x=(1:ℤ);l y=(2:ℤ);x+y\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('type ascription in function arguments', () => {
    const code = `λadd(a:ℤ,b:ℤ)→ℤ=a+b\nλf()→ℤ=add((3:ℤ),(4:ℤ))\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('const with type ascription', () => {
    const code = `c answer=(42:ℤ)\nλf()→ℤ=answer\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('rejects old const syntax', () => {
    const code = `c answer:ℤ=42\nλf()→ℤ=answer\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, false);
  });

  test('empty list in let binding', () => {
    const code = `λf()→[ℤ]=l xs=([]:[ℤ]);xs\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('list type ascription', () => {
    const code = `λf()→[ℤ]=([1,2,3]:[ℤ])\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('type ascription with negative int', () => {
    const code = `λf()→ℤ=(-5:ℤ)\n`;
    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });
});
