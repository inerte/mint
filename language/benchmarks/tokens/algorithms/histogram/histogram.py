Bucket = dict[str, int]


def buckets(histogram: dict[int, int]) -> list[Bucket]:
    return [{"count": value, "value": key} for key, value in histogram.items()]


def histogram(xs: list[int]) -> dict[int, int]:
    result: dict[int, int] = {}
    for value in xs:
        result = increment(result, value)
    return result


def increment(histogram: dict[int, int], value: int) -> dict[int, int]:
    next_histogram = dict(histogram)
    if value in next_histogram:
        next_histogram[value] = next_histogram[value] + 1
    else:
        next_histogram[value] = 1
    return next_histogram
