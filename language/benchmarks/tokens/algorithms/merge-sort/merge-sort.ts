function firstRun(runs: number[][]): number[] {
  if (runs.length === 0) return [];
  const [head, ...tail] = runs;
  if (tail.length === 0) return head;
  return head;
}

function merge(left: number[], right: number[]): number[] {
  if (left.length === 0) return right;
  const [leftHead, ...leftTail] = left;
  if (right.length === 0) return left;
  const [rightHead, ...rightTail] = right;
  if (leftHead <= rightHead) return [leftHead, ...merge(leftTail, right)];
  return [rightHead, ...merge(left, rightTail)];
}

function mergeAllRuns(runs: number[][]): number[] {
  return mergeAllRunsCount(runs.length, runs);
}

function mergeAllRunsCount(runCount: number, runs: number[][]): number[] {
  if (runCount <= 1) return firstRun(runs);
  return mergeAllRunsCount(nextRunCount(runCount), mergePairs(runs));
}

function mergePairs(runs: number[][]): number[][] {
  if (runs.length === 0) return [];
  const [head, ...tail] = runs;
  if (tail.length === 0) return wrapRun(head);
  const [right, ...rest] = tail;
  return [...wrapRun(merge(head, right)), ...mergePairs(rest)];
}

function mergeSort(xs: number[]): number[] {
  return mergeAllRuns(singletons(xs));
}

function nextRunCount(runCount: number): number {
  if (runCount === 0) return 0;
  if (runCount === 1) return 1;
  return 1 + nextRunCount(runCount - 2);
}

function singletons(xs: number[]): number[][] {
  return xs.map((head) => [head]);
}

function wrapAll(partials: number[][], values: number[]): number[][] {
  if (values.length === 0) return partials;
  const [head, ...tail] = values;
  return wrapAll(wrapHead(head, partials), tail);
}

function wrapHead(head: number, partials: number[][]): number[][] {
  if (partials.length === 0) return [];
  const [partial, ...tail] = partials;
  return [[...partial, head], ...wrapHead(head, tail)];
}

function wrapRun(run: number[]): number[][] {
  return wrapAll([[]], run);
}
