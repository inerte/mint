WordBucket = dict[str, int | str]


def frequency(lines: list[str]) -> dict[str, int]:
    result: dict[str, int] = {}
    for word in normalized_words(lines):
        result = increment(result, word)
    return result


def increment(histogram: dict[str, int], word: str) -> dict[str, int]:
    next_histogram = dict(histogram)
    if word in next_histogram:
        next_histogram[word] = next_histogram[word] + 1
    else:
        next_histogram[word] = 1
    return next_histogram


def non_empty(word: str) -> bool:
    return word != ""


def normalized_words(lines: list[str]) -> list[str]:
    return [word for line in lines for word in split_words(line) if non_empty(word)]


def split_words(line: str) -> list[str]:
    return line.lower().split(" ")


def summary(lines: list[str]) -> list[WordBucket]:
    return [{"count": value, "word": key} for key, value in frequency(lines).items()]
