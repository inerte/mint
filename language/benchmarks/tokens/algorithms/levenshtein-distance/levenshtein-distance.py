def levenshtein(left: str, right: str) -> int:
    return levenshtein_from(left, 0, right, 0)


def levenshtein_from(left: str, left_idx: int, right: str, right_idx: int) -> int:
    if left_idx == len(left):
        return len(right) - right_idx
    if right_idx == len(right):
        return len(left) - left_idx
    if left[left_idx] == right[right_idx]:
        return levenshtein_from(left, left_idx + 1, right, right_idx + 1)
    return 1 + min3(
        levenshtein_from(left, left_idx + 1, right, right_idx),
        levenshtein_from(left, left_idx, right, right_idx + 1),
        levenshtein_from(left, left_idx + 1, right, right_idx + 1),
    )


def min2(a: int, b: int) -> int:
    if a <= b:
        return a
    return b


def min3(a: int, b: int, third: int) -> int:
    return min2(min2(a, b), third)
