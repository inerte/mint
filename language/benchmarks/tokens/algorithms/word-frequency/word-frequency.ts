type WordBucket = { count: number; word: string };

function frequency(lines: string[]): Map<string, number> {
  return normalizedWords(lines).reduce(increment, new Map<string, number>());
}

function increment(histogram: Map<string, number>, word: string): Map<string, number> {
  const nextHistogram = new Map(histogram);
  if (nextHistogram.has(word)) {
    nextHistogram.set(word, nextHistogram.get(word)! + 1);
  } else {
    nextHistogram.set(word, 1);
  }
  return nextHistogram;
}

function nonEmpty(word: string): boolean {
  return word !== "";
}

function normalizedWords(lines: string[]): string[] {
  return lines.flatMap(splitWords).filter(nonEmpty);
}

function splitWords(line: string): string[] {
  return line.toLowerCase().split(" ");
}

function summary(lines: string[]): WordBucket[] {
  return Array.from(frequency(lines).entries()).map(([key, value]) => ({ count: value, word: key }));
}
