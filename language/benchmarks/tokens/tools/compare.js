#!/usr/bin/env node
/**
 * Benchmark Comparison Tool
 *
 * Compares published benchmark cases across languages.
 *
 * Usage:
 *   node language/benchmarks/tokens/tools/compare.js factorial
 */
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { compareImplementations, generateComparisonTable, calculateEfficiency } from './count-tokens.js';
const toolsDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(toolsDir, '..', '..', '..', '..');
const casesManifestPath = path.resolve(toolsDir, '..', 'cases.json');

function loadCasesManifest() {
    return JSON.parse(fs.readFileSync(casesManifestPath, 'utf8'));
}

function resolveCaseId(input) {
    if (!input) {
        return null;
    }
    if (fs.existsSync(input)) {
        return path.basename(path.resolve(input));
    }
    return input;
}

function findImplementationsFromManifest(input) {
    const cases = loadCasesManifest();
    const caseId = resolveCaseId(input);
    const benchmarkCase = caseId ? cases[caseId] : null;
    if (!benchmarkCase) {
        return null;
    }
    return {
        algorithmName: caseId,
        files: [
            benchmarkCase.python,
            benchmarkCase.sigil,
            benchmarkCase.typescript
        ].map((relativePath) => path.resolve(repoRoot, relativePath))
    };
}

function findImplementations(algorithmDir) {
    const files = [];
    if (!fs.existsSync(algorithmDir)) {
        throw new Error(`Directory not found: ${algorithmDir}`);
    }
    const entries = fs.readdirSync(algorithmDir);
    for (const entry of entries) {
        const fullPath = path.join(algorithmDir, entry);
        const ext = path.extname(entry);
        if (['.sigil', '.ts', '.py', '.rs', '.hs'].includes(ext)) {
            files.push(fullPath);
        }
    }
    return files;
}
function main() {
    const args = process.argv.slice(2);
    if (args.length === 0) {
        console.error('Usage: compare.js <case-id-or-case-directory>');
        console.error('Example: compare.js factorial');
        process.exit(1);
    }
    try {
        const manifestImplementations = findImplementationsFromManifest(args[0]);
        const algorithmName = manifestImplementations?.algorithmName ?? path.basename(args[0]);
        const files = manifestImplementations?.files ?? findImplementations(args[0]);
        console.log(`\n# ${algorithmName} - Token Comparison\n`);
        if (files.length === 0) {
            console.error('No implementation files found');
            process.exit(1);
        }
        console.log(`Found ${files.length} implementation(s):\n`);
        files.forEach(f => console.log(`  - ${path.basename(f)}`));
        console.log();
        const results = compareImplementations(files);
        const table = generateComparisonTable(results);
        const efficiency = calculateEfficiency(results);
        console.log('## Metrics\n');
        console.log(table);
        console.log('\n## Efficiency (vs TypeScript baseline)\n');
        console.log('| Language | Efficiency | Interpretation |');
        console.log('|----------|------------|----------------|');
        for (const [lang, eff] of efficiency) {
            const pct = ((eff - 1) * 100).toFixed(1);
            const interpretation = eff > 1
                ? `${pct}% more compact`
                : eff < 1
                    ? `${Math.abs(parseFloat(pct))}% more verbose`
                    : 'baseline';
            console.log(`| ${lang} | ${eff.toFixed(3)} | ${interpretation} |`);
        }
        console.log();
    }
    catch (error) {
        console.error('Error:', error);
        process.exit(1);
    }
}
main();
