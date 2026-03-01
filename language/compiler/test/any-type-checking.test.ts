import { describe, test } from 'node:test';
import assert from 'node:assert';
import { compileFromString } from '../src/api.js';

describe('Any Type Checking', () => {
  test('rejects specific record type when Any expected without type ascription', () => {
    // Minimal test: just the type and function, use .lib.sigil semantics
    const code = `t Response={body:𝕊, headers:Any, status:ℤ}

λjson(body:𝕊, status:ℤ)→Response={
  body:body,
  headers:{"Content-Type":"application/json"},
  status:status
}
`;

    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, false);
    if (!result.ok) {
      assert.strictEqual(result.error.code, 'SIGIL-TYPE-ERROR');
      assert.match(result.error.message, /Type mismatch.*expected any.*got record/i);
    }
  });

  test('accepts Any type when explicitly ascribed', () => {
    const code = `t Response={body:𝕊, headers:Any, status:ℤ}

λjson(body:𝕊, status:ℤ)→Response={
  body:body,
  headers:({"Content-Type":"application/json"}:Any),
  status:status
}
`;

    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('accepts Any value for Any field', () => {
    const code = `t Response={body:𝕊, headers:Any, status:ℤ}

λwrap(h:Any)→Response={
  body:"ok",
  headers:h,
  status:200
}
`;

    const result = compileFromString(code, 'test.lib.sigil');
    assert.strictEqual(result.ok, true);
  });

  test('rejects string when Any expected without type ascription', () => {
    // Try calling test with a string literal
    const code = `λmain()→𝕊=test("hello")
λtest(x:Any)→𝕊="ok"
`;

    const result = compileFromString(code);
    assert.strictEqual(result.ok, false);
    if (!result.ok) {
      assert.strictEqual(result.error.code, 'SIGIL-TYPE-ERROR');
    }
  });

  test('accepts string for Any when explicitly ascribed', () => {
    const code = `λmain()→𝕊=test(("hello":Any))
λtest(x:Any)→𝕊="ok"
`;

    const result = compileFromString(code);
    assert.strictEqual(result.ok, true);
  });
});
