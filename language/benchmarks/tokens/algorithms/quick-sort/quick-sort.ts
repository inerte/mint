function greaterOrEqual(pivot: number, xs: number[]): number[] {
  return xs.filter((value) => value >= pivot);
}

function lessThan(pivot: number, xs: number[]): number[] {
  return xs.filter((value) => value < pivot);
}

function quickSort(xs: number[]): number[] {
  return quickSortInto([], xs);
}

function quickSortInto(acc: number[], xs: number[]): number[] {
  if (xs.length === 0) return acc;
  const [pivot, ...tail] = xs;
  return quickSortInto([pivot, ...quickSortInto(acc, greaterOrEqual(pivot, tail))], lessThan(pivot, tail));
}
