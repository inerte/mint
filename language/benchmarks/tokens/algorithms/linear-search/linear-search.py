def linear_search(target: int, xs: list[int]) -> int:
    return linear_search_from(0, target, xs)


def linear_search_from(idx: int, target: int, xs: list[int]) -> int:
    if len(xs) == 0:
        return -1
    head, *tail = xs
    if head == target:
        return idx
    return linear_search_from(idx + 1, target, tail)
