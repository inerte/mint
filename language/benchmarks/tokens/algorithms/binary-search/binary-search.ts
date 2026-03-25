function binarySearch(high: number, low: number, target: number, xs: number[]): number {
  if (high < low) return -1;
  const mid = (low + high - (low + high) % 2) / 2;
  if (xs[mid] === target) return mid;
  if (xs[mid] < target) return binarySearch(high, mid + 1, target, xs);
  return binarySearch(mid - 1, low, target, xs);
}

function main(): number {
  return binarySearch(9, 0, 13, [1, 3, 5, 7, 9, 11, 13, 15, 17, 19]);
}
