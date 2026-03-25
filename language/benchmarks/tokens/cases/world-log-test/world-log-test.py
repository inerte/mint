from io_world import println
from test_log import contains
from world_log import capture


def main() -> None:
    pass


def test_worlds_capture_logs() -> bool:
    log = capture()
    println(log, "temporary")
    return contains(log, "temporary")


def test_worlds_start_fresh() -> bool:
    log = capture()
    return not contains(log, "temporary")
