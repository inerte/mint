type Bucket = { count: number; value: number };

function buckets(histogram: Map<number, number>): Bucket[] {
  return Array.from(histogram.entries()).map(([key, value]) => ({ count: value, value: key }));
}

function histogram(xs: number[]): Map<number, number> {
  return xs.reduce(increment, new Map<number, number>());
}

function increment(histogram: Map<number, number>, value: number): Map<number, number> {
  const nextHistogram = new Map(histogram);
  if (nextHistogram.has(value)) {
    nextHistogram.set(value, nextHistogram.get(value)! + 1);
  } else {
    nextHistogram.set(value, 1);
  }
  return nextHistogram;
}
