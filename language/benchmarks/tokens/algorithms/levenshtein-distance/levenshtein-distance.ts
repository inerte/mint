function levenshtein(left: string, right: string): number {
  return levenshteinFrom(left, 0, right, 0);
}

function levenshteinFrom(left: string, leftIdx: number, right: string, rightIdx: number): number {
  if (leftIdx === left.length) return right.length - rightIdx;
  if (rightIdx === right.length) return left.length - leftIdx;
  if (left.charAt(leftIdx) === right.charAt(rightIdx)) {
    return levenshteinFrom(left, leftIdx + 1, right, rightIdx + 1);
  }
  return 1 + min3(
    levenshteinFrom(left, leftIdx + 1, right, rightIdx),
    levenshteinFrom(left, leftIdx, right, rightIdx + 1),
    levenshteinFrom(left, leftIdx + 1, right, rightIdx + 1)
  );
}

function min2(a: number, b: number): number {
  if (a <= b) return a;
  return b;
}

function min3(a: number, b: number, third: number): number {
  return min2(min2(a, b), third);
}
