def binary_search(high: int, low: int, target: int, xs: list[int]) -> int:
    if high < low:
        return -1
    mid = (low + high - (low + high) % 2) // 2
    if xs[mid] == target:
        return mid
    if xs[mid] < target:
        return binary_search(high, mid + 1, target, xs)
    return binary_search(mid - 1, low, target, xs)


def main() -> int:
    return binary_search(9, 0, 13, [1, 3, 5, 7, 9, 11, 13, 15, 17, 19])
