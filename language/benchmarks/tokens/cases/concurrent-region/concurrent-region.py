import asyncio

ConcurrentOutcome = dict[str, int | str]
Result = tuple[bool, int | str]


async def sleep_ms(ms: int) -> None:
    await asyncio.sleep(ms / 1000)


async def main() -> list[ConcurrentOutcome]:
    return await asyncio.gather(*(run_process(value) for value in [1, 2, 3]))


async def process(value: int) -> Result:
    await sleep_ms(0)
    return (True, value)


async def run_process(value: int) -> ConcurrentOutcome:
    ok, payload = await process(value)
    return {"kind": "success", "value": payload} if ok else {"kind": "failure", "message": payload}
