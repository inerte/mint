import assert from 'assert';
import { buildBoundarySnippet, rewriteFileForTokenType } from './sigil-rewrite.js';
import { measureStringWithAllTokenizers } from './tokenizers.js';

const lambdaFile = {
  source: 'λends_with(s:𝕊,suffix:𝕊)→𝔹=⊥\n',
  tokens: [
    { type: 'LAMBDA', start: { offset: 0, index: 0 }, end: { offset: 1, index: 1 } },
    { type: 'IDENTIFIER', start: { offset: 1, index: 1 }, end: { offset: 10, index: 10 } },
    { type: 'LPAREN', start: { offset: 10, index: 10 }, end: { offset: 11, index: 11 } }
  ],
  symbols: [
    { type: 'LAMBDA', start: { offset: 0, index: 0 }, end: { offset: 1, index: 1 } }
  ]
};

const boolFile = {
  source: 'λmain()→𝔹=⊤∧⊥\n',
  tokens: [
    { type: 'LAMBDA', start: { offset: 0, index: 0 }, end: { offset: 1, index: 1 } },
    { type: 'IDENTIFIER', start: { offset: 1, index: 1 }, end: { offset: 5, index: 5 } },
    { type: 'LPAREN', start: { offset: 5, index: 5 }, end: { offset: 6, index: 6 } },
    { type: 'RPAREN', start: { offset: 6, index: 6 }, end: { offset: 7, index: 7 } },
    { type: 'ARROW', start: { offset: 7, index: 7 }, end: { offset: 8, index: 8 } },
    { type: 'TypeBool', start: { offset: 8, index: 8 }, end: { offset: 9, index: 10 } },
    { type: 'EQUAL', start: { offset: 9, index: 10 }, end: { offset: 10, index: 11 } },
    { type: 'TRUE', start: { offset: 10, index: 11 }, end: { offset: 11, index: 12 } },
    { type: 'AND', start: { offset: 11, index: 12 }, end: { offset: 12, index: 13 } },
    { type: 'FALSE', start: { offset: 12, index: 13 }, end: { offset: 13, index: 14 } }
  ],
  symbols: [
    { type: 'TRUE', start: { offset: 10, index: 11 }, end: { offset: 11, index: 12 } },
    { type: 'FALSE', start: { offset: 12, index: 13 }, end: { offset: 13, index: 14 } }
  ]
};

const rewrittenLambda = rewriteFileForTokenType(lambdaFile, 'LAMBDA', 'function');
assert.equal(rewrittenLambda, 'function ends_with(s:𝕊,suffix:𝕊)→𝔹=⊥\n');

const rewrittenTrue = rewriteFileForTokenType(boolFile, 'TRUE', 'true');
assert.equal(rewrittenTrue, 'λmain()→𝔹=true∧⊥\n');

const snippet = buildBoundarySnippet(lambdaFile, lambdaFile.symbols[0], 'fn', 8);
assert.equal(snippet.before, 'λends_wit');
assert.equal(snippet.after, 'fn ends_wit');

const exactCounts = measureStringWithAllTokenizers('⊤');
assert.ok(exactCounts.openai_cl100k_base >= 1);

console.log('unicode-benchmark tests passed');
