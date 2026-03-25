def insert_sorted(item: int, xs: list[int]) -> list[int]:
    if len(xs) == 0:
        return [item]
    head, *tail = xs
    if item <= head:
        return [item, head] + tail
    return [head] + insert_sorted(item, tail)


def insertion_sort(xs: list[int]) -> list[int]:
    if len(xs) == 0:
        return []
    head, *tail = xs
    return insert_sorted(head, insertion_sort(tail))
