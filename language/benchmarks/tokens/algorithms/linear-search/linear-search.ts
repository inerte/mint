function linearSearch(target: number, xs: number[]): number {
  return linearSearchFrom(0, target, xs);
}

function linearSearchFrom(idx: number, target: number, xs: number[]): number {
  if (xs.length === 0) return -1;
  const [head, ...tail] = xs;
  if (head === target) return idx;
  return linearSearchFrom(idx + 1, target, tail);
}
