def greater_or_equal(pivot: int, xs: list[int]) -> list[int]:
    return [value for value in xs if value >= pivot]


def less_than(pivot: int, xs: list[int]) -> list[int]:
    return [value for value in xs if value < pivot]


def quick_sort(xs: list[int]) -> list[int]:
    return quick_sort_into([], xs)


def quick_sort_into(acc: list[int], xs: list[int]) -> list[int]:
    if len(xs) == 0:
        return acc
    pivot, *tail = xs
    return quick_sort_into([pivot] + quick_sort_into(acc, greater_or_equal(pivot, tail)), less_than(pivot, tail))
