function insertSorted(item: number, xs: number[]): number[] {
  if (xs.length === 0) return [item];
  const [head, ...tail] = xs;
  if (item <= head) return [item, head, ...tail];
  return [head, ...insertSorted(item, tail)];
}

function insertionSort(xs: number[]): number[] {
  if (xs.length === 0) return [];
  const [head, ...tail] = xs;
  return insertSorted(head, insertionSort(tail));
}
