def first_run(runs: list[list[int]]) -> list[int]:
    if len(runs) == 0:
        return []
    head, *tail = runs
    if len(tail) == 0:
        return head
    return head


def merge(left: list[int], right: list[int]) -> list[int]:
    if len(left) == 0:
        return right
    left_head, *left_tail = left
    if len(right) == 0:
        return left
    right_head, *right_tail = right
    if left_head <= right_head:
        return [left_head] + merge(left_tail, right)
    return [right_head] + merge(left, right_tail)


def merge_all_runs(runs: list[list[int]]) -> list[int]:
    return merge_all_runs_count(len(runs), runs)


def merge_all_runs_count(run_count: int, runs: list[list[int]]) -> list[int]:
    if run_count <= 1:
        return first_run(runs)
    return merge_all_runs_count(next_run_count(run_count), merge_pairs(runs))


def merge_pairs(runs: list[list[int]]) -> list[list[int]]:
    if len(runs) == 0:
        return []
    head, *tail = runs
    if len(tail) == 0:
        return wrap_run(head)
    right, *rest = tail
    return wrap_run(merge(head, right)) + merge_pairs(rest)


def merge_sort(xs: list[int]) -> list[int]:
    return merge_all_runs(singletons(xs))


def next_run_count(run_count: int) -> int:
    if run_count == 0:
        return 0
    if run_count == 1:
        return 1
    return 1 + next_run_count(run_count - 2)


def singletons(xs: list[int]) -> list[list[int]]:
    return [[head] for head in xs]


def wrap_all(partials: list[list[int]], values: list[int]) -> list[list[int]]:
    if len(values) == 0:
        return partials
    head, *tail = values
    return wrap_all(wrap_head(head, partials), tail)


def wrap_head(head: int, partials: list[list[int]]) -> list[list[int]]:
    if len(partials) == 0:
        return []
    partial, *tail = partials
    return [partial + [head]] + wrap_head(head, tail)


def wrap_run(run: list[int]) -> list[list[int]]:
    return wrap_all([[]], run)
