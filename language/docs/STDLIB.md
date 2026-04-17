# Sigil Standard Library

## Overview

The Sigil standard library provides core utility functions and predicates for common programming tasks. All functions follow canonical form principles - exactly ONE way to solve each problem.

## Current Status

**Implemented:**
- ✅ Decode / validation pipeline for trusted internal data - `stdlib/decode`
- ✅ List predicates (validation, checking) - `stdlib/list`
- ✅ Numeric predicates and ranges - `stdlib/numeric`
- ✅ List utilities (head, tail, take/drop/reverse, safe lookup) - `stdlib/list`
- ✅ String operations (manipulation, searching) - `stdlib/string`
- ✅ String predicates (prefix/suffix checking) - `stdlib/string`
- ✅ File system operations - `stdlib/file`
- ✅ Filesystem watch streams - `stdlib/fsWatch`
- ✅ Process execution for harnesses and tooling - `stdlib/process`
- ✅ PTY-backed interactive sessions - `stdlib/pty`
- ✅ Random number generation and collection helpers - `stdlib/random`
- ✅ Regular-expression compile/test/search with all-matches support - `stdlib/regex`
- ✅ Float arithmetic and math functions - `stdlib/float`
- ✅ Cryptographic hashing and encoding - `stdlib/crypto`
- ✅ HTTP and TCP clients and servers - `stdlib/httpClient`, `stdlib/httpServer`, `stdlib/tcpClient`, `stdlib/tcpServer`
- ✅ WebSocket servers and route-scoped text streams - `stdlib/websocket`
- ✅ Runtime dependency topology - `stdlib/topology`
- ✅ Runtime dependency config helpers - `stdlib/config`
- ✅ JSON parsing/serialization - `stdlib/json`
- ✅ Path manipulation - `stdlib/path`
- ✅ Pull-based event sources - `stdlib/stream`
- ✅ Time parsing/comparison/clock - `stdlib/time`
- ✅ Terminal raw-mode input and cursor control - `stdlib/terminal`
- ✅ URL parsing/query helpers - `stdlib/url`
- ✅ Deterministic feature-flag evaluation - `stdlib/featureFlags`
- ✅ Core prelude vocabulary (Option, Result) - `core/prelude` (implicit)
- ✅ Length operator (`#`) - works on strings, lists, and maps

## Rooted Module Syntax

```sigil program
e console

λmain()=>Unit=console.log(§string.intToString(#[
  1,
  2,
  3
])
  ++" "
  ++§time.formatIso(§time.fromEpochMillis(0)))
```

**Design:** Sigil writes rooted references directly at the use site.
There are no import declarations, no selective imports, and no aliases. FFI
still uses `e module::path`; Sigil modules use roots like `§`, `•`, `¶`, `¤`,
`†`, `※`, and `☴`, while project-defined types and project sum constructors use
`µ`.

## Length Operator (`#`)

The `#` operator is a **built-in language operator** that returns the length of strings, lists, and maps.

**Syntax:**
```text
#expression => Int
```

**Type Checking:**
- Works on strings (`String`), lists (`[T]`), and maps (`{K↦V}`)
- Compile error for other types
- Always returns integer (`Int`)

**Examples:**
```sigil program
λmain()=>Bool=#"hello"=5
  and #""=0
  and #[
    1,
    2,
    3
  ]=3
  and #{
    "a"↦1,
    "b"↦2
  }=2
```

**Note on Empty Lists:**
Empty lists `[]` infer their type from context:
- In pattern matching: First arm establishes the type
- In function return: Return type annotation provides context
- In standalone expressions: Type cannot be inferred (use function with explicit return type)

**Why `#` instead of functions?**

1. **ONE canonical form** - Not `§string` helper calls vs `§list` helper calls, just `#`
2. **Leverages bidirectional type checking** - Type is known at compile time
3. **Concise** - Machine-first language optimizes for brevity (`#s` vs `len(s)`)
4. **Zero syntactic variation** - Single way to express "get length"

**Codegen:**
```typescript
#s          => (await s).length
#[1,2,3]    => (await [1,2,3]).length
#{"a"↦1}    => (await new Map([["a",1]])).size
```

**Note:** The deprecated `§list.len` function has been removed. Use `#` instead.

## Module Exports

Sigil uses file-based visibility:
- `.lib.sigil` exports all top-level declarations automatically
- `.sigil` files are executable-oriented

There is no `export` keyword.

## Feature Flags

`§featureFlags` is the canonical typed evaluation surface for first-class
`featureFlag` declarations.

Current public types:

```sigil decl §featureFlags
t Config[T,C]={key:Option[λ(C)=>Option[String]],rules:[Rule[T,C]]}
t Entry[C]
t Flag[T]={createdAt:String,default:T,id:String}
t RolloutPlan[T]={percentage:Int,variants:[WeightedValue[T]]}
t Rule[T,C]={action:RuleAction[T],predicate:λ(C)=>Bool}
t RuleAction[T]=Rollout(RolloutPlan[T])|Value(T)
t Set[C]=[Entry[C]]
t WeightedValue[T]={value:T,weight:Int}

λentry[C,T](config:Config[T,C],flag:Flag[T])=>Entry[C]
λget[C,T](context:C,flag:Flag[T],set:Set[C])=>T
```

Canonical usage:

```sigil expr
§featureFlags.get(
  context,
  ☴featureFlagStorefrontFlags::flags.NewCheckout,
  •config.flags
)
```

Current `§featureFlags.get` precedence is:

1. first matching rule wins
2. `Value(...)` returns its value immediately
3. `Rollout(...)` deterministically buckets with the resolved key
4. if no rule matches, return the declaration `default`

`Entry[C]` and `Set[C]` let one config snapshot hold multiple flag value types
while keeping the context type explicit.

## File, FsWatch, Path, Process, Pty, Stream, WebSocket, Random, JSON, Time, and URL

`§file` exposes canonical UTF-8 filesystem helpers:

```sigil program
λmain()=>!Fs String={
  l out=(§path.join(
    "/tmp",
    "sigil.txt"
  ):String);
  l _=(§file.writeText(
    "hello",
    out
  ):Unit);
  §file.readText(out)
}
```

It also exposes `makeTempDir(prefix)` for canonical temp workspace creation in
tooling and harness code.

For topology-aware projects with labelled boundary handling, the named-boundary
surface is:

- `appendTextAt`
- `existsAt`
- `listDirAt`
- `makeDirAt`
- `makeDirsAt`
- `makeTempDirAt`
- `readTextAt`
- `removeAt`
- `removeTreeAt`
- `writeTextAt`

Those functions take a `§topology.FsRoot` handle so policies can target exact
filesystem roots.

`§fsWatch` exposes canonical advisory recursive filesystem watches backed by
`§stream`:

```sigil decl §fsWatch
t Event=Changed(String)|Created(String)|Removed(String)
t Watch={id:String}

λclose(watch:Watch)=>!FsWatch Unit
λevents(watch:Watch)=>!FsWatch §stream.Source[Event]
λwatch(path:String)=>!FsWatch Owned[Watch]
λwatchAt(path:String,root:§topology.FsRoot)=>!FsWatch Owned[Watch]
```

FsWatch rules:
- watches are recursive in v1
- emitted paths are relative to the watched directory
- events are advisory; duplicate or coalesced delivery is allowed
- `watch` and `watchAt` return owned watch handles and are intended to be used with `using`
- `watchAt` is the named-boundary variant for topology-aware projects and takes a `§topology.FsRoot`
- rename detection is not modeled separately in v1

`§path` exposes canonical filesystem path operations:

```sigil program
λmain()=>[String]=[
  §path.basename("website/articles/hello.md"),
  §path.join(
    "website",
    "articles"
  )
]
```

`§process` exposes canonical argv-based child-process execution:

```sigil program
λmain()=>!Process Unit={
  l result=(§process.run(§process.command([
    "git",
    "status"
  ])):§process.ProcessResult);
  match result.code=0{
    true=>()|
    false=>()
  }
}
```

The canonical process surface is:
- `command`
- `exit`
- `withCwd`
- `withEnv`
- `run`
- `runAt`
- `runChecked`
- `runJson`
- `start`
- `startAt`
- `wait`
- `kill`

Commands are argv-based only. Non-zero exit status is returned in
`ProcessResult.code`; it is not a separate failure channel. When a caller wants
checked failure semantics, use:

- `runChecked(command)=>Result[ProcessResult,ProcessFailure]`
- `runJson(command)=>Result[§json.JsonValue,ProcessFailure]`

`start` and `startAt` return owned process handles and are intended to be used
with `using`.

`§pty` exposes canonical interactive PTY sessions backed by `§stream`:

```sigil decl §pty
t Event=Output(String)|Exit(Int)
t Session={pid:Int}
t SessionRef={id:String}
t Spawn={argv:[String],cols:Int,cwd:Option[String],env:{String↦String},rows:Int}

λclose(session:Session)=>!Pty Unit
λcloseManaged(session:SessionRef)=>!Pty Unit
λevents(session:Session)=>!Pty §stream.Source[Event]
λeventsManaged(session:SessionRef)=>!Pty Owned[§stream.Source[Event]]
λresize(cols:Int,rows:Int,session:Session)=>!Pty Unit
λresizeManaged(cols:Int,rows:Int,session:SessionRef)=>!Pty Unit
λspawn(request:Spawn)=>!Pty Owned[Session]
λspawnManaged(request:Spawn)=>!Pty SessionRef
λspawnAt(handle:§topology.PtyHandle,request:Spawn)=>!Pty Owned[Session]
λspawnManagedAt(handle:§topology.PtyHandle,request:Spawn)=>!Pty SessionRef
λwait(session:Session)=>!Pty Int
λwaitManaged(session:SessionRef)=>!Pty Int
λwrite(input:String,session:Session)=>!Pty Unit
λwriteManaged(input:String,session:SessionRef)=>!Pty Unit
```

PTY rules:
- `events` exposes one combined terminal stream
- `Output(text)` carries terminal chunks in arrival order
- `Exit(code)` is emitted once when the session terminates
- `wait` resolves to the same exit code reported by the session
- `spawn` and `spawnAt` return owned session handles and are intended to be used with `using`
- `spawnManaged` and `spawnManagedAt` return storable runtime-managed session refs for long-lived server state
- `eventsManaged` returns an owned subscription stream for one managed session ref
- `closeManaged` is idempotent
- `spawnAt` is the named-boundary variant for topology-aware projects and takes a `§topology.PtyHandle`
- `spawnManagedAt` is the named-boundary managed-ref variant for topology-aware projects and takes a `§topology.PtyHandle`

`§stream` exposes canonical pull-based runtime event sources:

```sigil decl §stream
t Hub[T]=StreamHub(Int)
t Next[T]=Done()|Item(T)
t Source[T]=StreamSource(Int)

λclose[T](source:Source[T])=>!Stream Unit
λhub[T]()=>!Stream Owned[Hub[T]]
λnext[T](source:Source[T])=>!Stream Next[T]
λpublish[T](hub:Hub[T],value:T)=>!Stream Unit
λsubscribe[T](hub:Hub[T])=>!Stream Owned[Source[T]]
```

Stream rules:
- `Source[T]` is the canonical handle returned by stream-backed runtime APIs
- `Hub[T]` is the canonical fanout surface for long-running app event distribution
- `next` yields `Item(value)` while values remain and `Done()` when the source is exhausted
- `close` is idempotent
- after `close`, subsequent `next` calls return `Done()`
- `hub` and `subscribe` return owned handles and are intended to be used with `using`
- `publish` fanouts to current subscribers in send order
- generic stream failure is not modeled in `§stream`; producer APIs own their error events
- `§stream` is intentionally small and does not expose combinator-style operator families

`§websocket` exposes canonical server-first WebSocket handling backed by
`§stream`:

```sigil decl §websocket
t Client={id:String}
t Route={handle:§topology.WebSocketHandle,path:String}
t Server={port:Int}

λclose(client:Client)=>!WebSocket Unit
λconnections(handle:§topology.WebSocketHandle,server:Server)=>!WebSocket Owned[§stream.Source[Client]]
λlisten(port:Int,routes:[Route])=>!WebSocket Owned[Server]
λmessages(client:Client)=>!WebSocket Owned[§stream.Source[String]]
λport(server:Server)=>Int
λroute(handle:§topology.WebSocketHandle,path:String)=>Route
λsend(client:Client,text:String)=>!WebSocket Unit
λwait(server:Server)=>!WebSocket Unit
```

WebSocket rules:
- `listen` binds one port plus an exact-path route list
- route paths must be unique within one server
- route handles must be unique within one server
- `connections` yields accepted clients for one exact `§topology.WebSocketHandle`
- `messages` yields text frames for one client
- `listen`, `connections`, and `messages` return owned handles and are intended to be used with `using`
- `send` writes one text frame to one client
- `close` closes one client connection
- v1 is server-only; there is no WebSocket client API, binary-frame surface, or broadcast helper

`runAt` and `startAt` are the named-boundary variants for topology-aware
projects. They take a `Command` plus a `§topology.ProcessHandle`.

`§log` is the named-boundary logging surface:

```sigil program projects/labelled-boundaries/src/logExample.sigil
λmain()=>!Log Unit=§log.write(
  "customer created",
  •topology.auditLog
)
```

It currently exposes:
- `write`

Projects can keep using `§io` for ordinary textual output, but labelled
boundary rules target `§log.write` because it names the sink explicitly.

`§random` exposes the canonical runtime random surface:

```sigil program
λmain()=>!Random Unit={
  l _=(§random.intBetween(
    6,
    1
  ):Int);
  l deck=(§random.shuffle([
    "orc",
    "slime",
    "bat"
  ]):[String]);
  l _=(§random.pick(deck):Option[String]);
  ()
}
```

The canonical random surface is:
- `intBetween`
- `pick`
- `shuffle`

Randomness is world-driven through `†random.real()`, `†random.seeded(seed)`,
and `†random.fixture(draws)`.

`§regex` exposes a small JavaScript-backed regular-expression surface:

```sigil program
λmain()=>String match §regex.compile(
  "i",
  "^(sigil)-(.*)$"
){
  Ok(regex)=>match §regex.find(
    "Sigil-lang",
    regex
  ){
    Some(found)=>found.full|
    None()=>""
  }|
  Err(_)=>""
}
```

The canonical regex surface is:
- `compile`
- `find`
- `findAll`
- `isMatch`

Regex semantics follow JavaScript `RegExp`, including pattern syntax and flags.
`compile` validates the pattern/flags first and returns `Err` on invalid input.
`find` returns the first match; `findAll` returns all non-overlapping matches as
a list. `findAll` automatically adds the `g` flag internally — callers do not
need to include it.

`§json` exposes a typed JSON AST with safe parsing:

```sigil program
λmain()=>Unit match §json.parse("{\"ok\":true}"){
  Ok(value)=>match §json.asObject(value){
    Some(_)=>()|
    None()=>()
  }|
  Err(_)=>()
}
```

`§decode` is the canonical layer for turning raw `JsonValue` into trusted
internal Sigil values:

```sigil module
t Message={
  createdAt:§time.Instant,
  text:String
}

λinstant(value:§json.JsonValue)=>Result[
  §time.Instant,
  §decode.DecodeError
] match §decode.string(value){
  Ok(text)=>match §time.parseIso(text){
    Ok(instant)=>Ok(instant)|
    Err(error)=>Err({
      message:error.message,
      path:[]
    })
  }|
  Err(error)=>Err(error)
}

λmessage(value:§json.JsonValue)=>Result[
  Message,
  §decode.DecodeError
] match §decode.field(
  instant,
  "createdAt"
)(value){
  Ok(createdAt)=>match §decode.field(
    §decode.string,
    "text"
  )(value){
    Ok(text)=>Ok({
      createdAt:createdAt,
      text:text
    })|
    Err(error)=>Err(error)
  }|
  Err(error)=>Err(error)
}
```

The intended split is:
- `§json` for raw parse / inspect / stringify
- `§decode` for decode / validate / trust

If a field may be absent, keep the record exact and use `Option[T]` in that
field. Sigil does not use open or partial records for this.

`§time` exposes strict ISO parsing, instant comparison, and harness sleep:

```sigil program
λmain()=>Int match §time.parseIso("2026-03-03"){
  Ok(instant)=>§time.toEpochMillis(instant)|
  Err(_)=>0
}
```

Effectful code may also use `§time.sleepMs(ms)` for retry loops and
process orchestration.

`§timer` exposes event-source timers for long-running app workflows:

```sigil decl §timer
λafterMs(ms:Int)=>!Timer Owned[§stream.Source[Unit]]
λeveryMs(ms:Int)=>!Timer Owned[§stream.Source[Unit]]
```

Timer rules:
- `afterMs` yields one `()` tick and then finishes
- `everyMs` yields repeated `()` ticks until the source is closed
- both functions return owned stream sources and are intended to be used with `using`

`§task` exposes cancellable background work:

```sigil decl §task
t Task[T]={id:Int}
t TaskResult[T]=Cancelled()|Failed(String)|Succeeded(T)

λcancel[T](task:Task[T])=>!Task Unit
λspawn[T](work:λ()=>T)=>!Task Owned[Task[T]]
λwait[T](task:Task[T])=>!Task TaskResult[T]
```

Task rules:
- `spawn` returns an owned task handle and is intended to be used with `using`
- `cancel` requests cancellation
- `wait` resolves to `Succeeded(value)`, `Cancelled()`, or `Failed(message)`

`§terminal` exposes a small raw-terminal surface for turn-based interactive
programs:

```sigil program
λmain()=>!Terminal Unit={
  l _=(§terminal.enableRawMode():Unit);
  l key=(§terminal.readKey():§terminal.Key);
  l _=(§terminal.disableRawMode():Unit);
  match key{
    §terminal.Text(text)=>()|
    §terminal.Escape()=>()
  }
}
```

The canonical terminal surface is:
- `clearScreen`
- `enableRawMode`
- `disableRawMode`
- `hideCursor`
- `showCursor`
- `readKey`
- `write`

`readKey` normalizes terminal input into `§terminal.Key`, currently:
- `Escape()`
- `Text(String)`

`§url` exposes strict parse results and typed URL fields for both absolute and relative targets:

```sigil program
λmain()=>[String] match §url.parse("../language/spec/cli-json.md?view=raw#schema"){
  Ok(url)=>[
    url.path,
    §url.suffix(url)
  ]|
  Err(_)=>[]
}
```

## HTTP Client and Server

`§httpClient` is the canonical text-based HTTP client layer.

For topology-aware projects, the canonical surface is handle-based rather than
raw-URL based:

```sigil program projects/topology-http/src/getClient.sigil
λmain()=>!Http String match §httpClient.get(
  •topology.mailerApi,
  §httpClient.emptyHeaders(),
  "/health"
){
  Ok(response)=>response.body|
  Err(error)=>error.message
}
```

The split is:
- transport/URL failures return `Err(HttpError)`
- any received HTTP response, including `404` and `500`, returns `Ok(HttpResponse)`
- JSON helpers compose over `§json`
- topology-aware application code must not pass raw base URLs directly

`§topology` owns the dependency handles.
`config/*.lib.sigil` now exports `world`, built through `†http`, `†tcp`, and `†runtime`.

`§httpServer` is the canonical request/response server layer. For simple
programs, `serve` remains available. For real app/server orchestration, the
canonical surface is request-stream based:

```sigil decl §httpServer
t Headers={String↦String}
t HttpBodyError={message:String}
t PendingRequest={request:Request,responder:Responder}
t Request={body:String,headers:Headers,method:String,path:String}
t Responder={id:String}
t Response={body:String,headers:Headers,status:Int}
t RouteMatch={params:{String↦String}}
t Server={port:Int}
t WebSocketClient={id:String}
t WebSocketRoute={handle:§topology.WebSocketHandle,path:String}

λjson(body:String,status:Int)=>Response
λjsonBody(request:Request)=>Result[§json.JsonValue,HttpBodyError]
λlisten(port:Int)=>!Http Owned[Server]
λlistenWithWebSockets(port:Int,routes:[WebSocketRoute])=>!Http Owned[Server]
λlistenWith(handler:λ(Request)=>Response,port:Int)=>!Http Server
λlogRequest(request:Request)=>!Log Unit
λmatch(method:String,pathPattern:String,request:Request)=>Option[RouteMatch]
λnotFound()=>Response
λnotFoundMsg(path:String)=>Response
λok(body:String)=>Response
λport(server:Server)=>Int
λreply(responder:Responder,response:Response)=>!Http Unit
λrequests(server:Server)=>!Http Owned[§stream.Source[PendingRequest]]
λresponse(body:String,contentType:String,status:Int)=>Response
λserve(handler:λ(Request)=>Response,port:Int)=>!Http Unit
λserverError(message:String)=>Response
λwait(server:Server)=>!Http Unit
λwebsocketClose(client:WebSocketClient)=>!Http Unit
λwebsocketConnections(handle:§topology.WebSocketHandle,server:Server)=>!Http Owned[§stream.Source[WebSocketClient]]
λwebsocketMessages(client:WebSocketClient)=>!Http Owned[§stream.Source[String]]
λwebsocketRoute(handle:§topology.WebSocketHandle,path:String)=>WebSocketRoute
λwebsocketSend(client:WebSocketClient,text:String)=>!Http Unit
```

The public server surface is:
- `listen`
- `listenWithWebSockets`
- `requests`
- `reply`
- `jsonBody`
- `match`
- `listenWith`
- `port`
- `serve`
- `wait`
- `websocketRoute`
- `websocketConnections`
- `websocketMessages`
- `websocketSend`
- `websocketClose`

`listen` returns an owned server handle. `requests(server)` opens an owned
request stream of `PendingRequest` values, and `reply` answers one pending
request through its `Responder`.

`listenWithWebSockets(port,routes)` returns one owned HTTP server handle that
also owns exact-path websocket upgrades on the same bound port. Use
`websocketRoute` to declare websocket upgrade paths and
`websocketConnections(handle,server)` / `websocketMessages(client)` to consume
the resulting connection and message streams.

`listenWith(handler,port)` and `serve(handler,port)` remain available for simple
pure-handler programs. The request-stream surface is the canonical app/server
surface for long-running Sigil apps because it composes with `using`, `§task`,
and `§stream`.

Passing `0` to `listen` or `serve` asks the OS for any free ephemeral port. Use
`§httpServer.port(server)` after `listen` when the actual port matters.

## TCP Client and Server

`§tcpClient` is the canonical one-request, one-response TCP client layer.

For topology-aware projects, the canonical surface is handle-based:

```sigil program projects/topology-tcp/src/pingClient.sigil
λmain()=>!Tcp String match §tcpClient.send(
  •topology.eventStream,
  "ping"
){
  Ok(response)=>response.message|
  Err(error)=>error.message
}
```

The canonical framing model is:
- UTF-8 text only
- one newline-delimited request per connection
- one newline-delimited response per connection

`§topology` owns the dependency handles.
`config/*.lib.sigil` now exports `world`, built through `†http`, `†tcp`, and `†runtime`.

`§tcpServer` is the matching minimal TCP server layer:

```sigil program
λhandle(request:§tcpServer.Request)=>§tcpServer.Response=§tcpServer.response(request.message)

λmain()=>!Tcp Unit=§tcpServer.serve(
  handle,
  45120
)
```

The public server surface is:
- `listen`
- `port`
- `serve`
- `wait`

`serve` remains the canonical blocking entrypoint for normal programs. `listen`
returns a `§tcpServer.Server` handle, `port` reports the actual bound port, and
`wait` blocks on that handle.

Passing `0` to `listen` or `serve` asks the OS for any free ephemeral port. Use
`§tcpServer.port(server)` after `listen` when the actual port matters.

## Topology

`§topology` is the canonical declaration layer for named runtime boundaries.
The canonical environment runtime layer now lives under the compiler-owned `†`
roots rather than `§config`.

`§config` remains available for low-level binding value helpers inside
config modules, but project environments no longer export `Bindings`. The env
ABI is `c world=(...:†runtime.World)`.

Topology-aware projects define `src/topology.lib.sigil`, `src/policies.lib.sigil`,
the selected `config/<env>.lib.sigil`, and use typed handles instead of raw
endpoints or ad hoc sink names in application code:

```sigil program projects/topology-http/src/getClient.sigil
λmain()=>!Http Unit match §httpClient.get(
  •topology.mailerApi,
  §httpClient.emptyHeaders(),
  "/health"
){
  Ok(_)=>()|
  Err(_)=>()
}
```

See [topology.md](./topology.md) for the full model.

## List Predicates

**Module:** `stdlib/list`

### sortedAsc

Check if a list is sorted in ascending order.

```sigil decl §list
λsortedAsc(xs:[Int])=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=§list.sortedAsc([
  1,
  2,
  3
])
  and ¬§list.sortedAsc([
    3,
    2,
    1
  ])
  and §list.sortedAsc([])
  and §list.sortedAsc([5])
```

**Use case:** Validate precondition for binary search or other sorted-list algorithms.

### sortedDesc

Check if a list is sorted in descending order.

```sigil decl §list
λsortedDesc(xs:[Int])=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=§list.sortedDesc([
  3,
  2,
  1
]) and ¬§list.sortedDesc([
  1,
  2,
  3
])
```

### all

Check if all elements in a list satisfy a predicate.

```sigil decl §list
λall[T](pred:λ(T)=>Bool,xs:[T])=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=§list.all(
  §numeric.isPositive,
  [
    1,
    2,
    3
  ]
)
  and ¬§list.all(
    §numeric.isPositive,
    [
      1,
      -2,
      3
    ]
  )
  and §list.all(
    §numeric.isEven,
    [
      2,
      4,
      6
    ]
  )
```

**Use case:** Validate that all elements meet a requirement.

### any

Check if any element in a list satisfies a predicate.

```sigil decl §list
λany[T](pred:λ(T)=>Bool,xs:[T])=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=¬§list.any(
  §numeric.isEven,
  [
    1,
    3,
    5
  ]
)
  and §list.any(
    §numeric.isEven,
    [
      1,
      2,
      3
    ]
  )
  and §list.any(
    §numeric.isPrime,
    [
      4,
      6,
      8,
      7
    ]
  )
```

**Use case:** Check if at least one element meets a requirement.

### contains

Check if an element exists in a list.

```sigil decl §list
λcontains[T](item:T,xs:[T])=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=§list.contains(
  3,
  [
    1,
    2,
    3,
    4
  ]
)
  and ¬§list.contains(
    5,
    [
      1,
      2,
      3,
      4
    ]
  )
  and ¬§list.contains(
    1,
    []
  )
```

**Use case:** Membership testing.

### count

Count occurrences of an element in a list.

```sigil decl §list
λcount[T](item:T,xs:[T])=>Int
```

### countIf

Count elements that satisfy a predicate.

```sigil decl §list
λcountIf[T](pred:λ(T)=>Bool,xs:[T])=>Int
```

### drop

Drop the first `n` elements.

```sigil decl §list
λdrop[T](n:Int,xs:[T])=>[T]
```

### find

Find the first element that satisfies a predicate.

```sigil decl §list
λfind[T](pred:λ(T)=>Bool,xs:[T])=>Option[T]
```

Examples:
```sigil program
λmain()=>Bool=(match §list.find(
  §numeric.isEven,
  [
    1,
    3,
    4,
    6
  ]
){
  Some(value)=>value=4|
  None()=>false
}) and (match §list.find(
  §numeric.isEven,
  [
    1,
    3,
    5
  ]
){
  Some(_)=>false|
  None()=>true
})
```

### flatMap

Map each element to a list and flatten the results in order.

```sigil decl §list
λflatMap[T,U](fn:λ(T)=>[U],xs:[T])=>[U]
```

Examples:
```sigil program
λmain()=>Bool=§list.flatMap(
  λ(x:Int)=>[Int]=[
    x,
    x
  ],
  [
    1,
    2,
    3
  ]
)=[
  1,
  1,
  2,
  2,
  3,
  3
]
```

### fold

Reduce a list to a single value by threading an accumulator from left to right.

```sigil decl §list
λfold[T,U](acc:U,fn:λ(U,T)=>U,xs:[T])=>U
```

Examples:
```sigil program
λappendDigit(acc:Int,x:Int)=>Int=acc*10+x

λmain()=>Bool=§list.fold(
  0,
  λ(acc:Int,x:Int)=>Int=acc+x,
  [
    1,
    2,
    3
  ]
)=6 and §list.fold(
  0,
  appendDigit,
  [
    1,
    2,
    3
  ]
)=123
```

### inBounds

Check if an index is valid for a list (in range [0, len-1]).

```sigil decl §list
λinBounds[T](idx:Int,xs:[T])=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=§list.inBounds(
  0,
  [
    1,
    2,
    3
  ]
)
  and §list.inBounds(
    2,
    [
      1,
      2,
      3
    ]
  )
  and ¬§list.inBounds(
    3,
    [
      1,
      2,
      3
    ]
  )
  and ¬§list.inBounds(
    -1,
    [
      1,
      2,
      3
    ]
  )
  and ¬§list.inBounds(
    0,
    []
  )
```

**Use case:** Validate array/list access before indexing. Prevents out-of-bounds errors.

**Implementation:** Uses `#xs` to check bounds.

## List Utilities

**Module:** `stdlib/list`

**Note:** Use the `#` operator for list length instead of a function (e.g., `#[1,2,3]` => `3`).

### last

Get the last element safely.

```sigil decl §list
λlast[T](xs:[T])=>Option[T]
```

Examples:
```sigil program
λmain()=>Bool=(match §list.last([]){
  Some(_)=>false|
  None()=>true
}) and (match §list.last([
  1,
  2,
  3
]){
  Some(value)=>value=3|
  None()=>false
})
```

### max

Get the maximum element safely.

```sigil decl §list
λmax(xs:[Int])=>Option[Int]
```

Examples:
```sigil program
λmain()=>Bool=(match §list.max([]){
  Some(_)=>false|
  None()=>true
}) and (match §list.max([
  3,
  9,
  4
]){
  Some(value)=>value=9|
  None()=>false
})
```

### min

Get the minimum element safely.

```sigil decl §list
λmin(xs:[Int])=>Option[Int]
```

Examples:
```sigil program
λmain()=>Bool=(match §list.min([]){
  Some(_)=>false|
  None()=>true
}) and (match §list.min([
  3,
  9,
  4
]){
  Some(value)=>value=3|
  None()=>false
})
```

### nth

Get the item at a zero-based index safely.

```sigil decl §list
λnth[T](idx:Int,xs:[T])=>Option[T]
```

Examples:
```sigil program
λmain()=>Bool=(match §list.nth(
  0,
  [
    7,
    8
  ]
){
  Some(value)=>value=7|
  None()=>false
}) and (match §list.nth(
  2,
  [
    7,
    8
  ]
){
  Some(_)=>false|
  None()=>true
})
```

### product

Multiply all integers in a list.

```sigil decl §list
λproduct(xs:[Int])=>Int
```

Examples:
```sigil program
λmain()=>Bool=§list.product([])=1 and §list.product([
  2,
  3,
  4
])=24
```

### removeFirst

Remove the first occurrence of an element.

```sigil decl §list
λremoveFirst[T](item:T,xs:[T])=>[T]
```

### reverse

Reverse a list.

```sigil decl §list
λreverse[T](xs:[T])=>[T]
```

### sum

Sum all integers in a list.

```sigil decl §list
λsum(xs:[Int])=>Int
```

Examples:
```sigil program
λmain()=>Bool=§list.sum([])=0 and §list.sum([
  1,
  2,
  3,
  4
])=10
```

### take

Take the first `n` elements.

```sigil decl §list
λtake[T](n:Int,xs:[T])=>[T]
```

## Numeric Helpers

**Module:** `stdlib/numeric`

### range

Build an ascending integer range, inclusive at both ends.

```sigil decl §numeric
λrange(start:Int,stop:Int)=>[Int]
```

Examples:
```sigil program
λmain()=>Bool=§numeric.range(
  2,
  5
)=[
  2,
  3,
  4,
  5
]
  and §numeric.range(
    3,
    3
  )=[3]
  and §numeric.range(
    5,
    2
  )=[]
```

## Canonical List-Processing Surface

For ordinary list work, Sigil expects the canonical operators and stdlib path,
not hand-rolled recursive plumbing:

- use `§list.all` for universal checks
- use `§list.any` for existential checks
- use `§list.countIf` for predicate counting
- use `map` for projection
- use `filter` for filtering
- use `§list.find` for first-match search
- use `§list.flatMap` for flattening projection
- use `reduce ... from ...` or `§list.fold` for reduction
- use `§list.reverse` for reversal

Sigil now rejects exact recursive clones of `all`, `any`, `map`, `filter`,
`find`, `flatMap`, `fold`, and `reverse`, rejects `#(xs filter pred)` in favor of
`§list.countIf`, and rejects recursive result-building of the form
`self(rest)⧺rhs`.

Outside `language/stdlib/`, Sigil also rejects exact top-level wrappers whose
body is already a canonical helper surface such as `§list.sum(xs)`,
`§numeric.max(a,b)`, `§string.trim(s)`, `xs map fn`, `xs filter pred`, or
`xs reduce fn from init`.
Call the canonical helper directly instead of renaming it.

## String Operations

**Module:** `stdlib/string`

Comprehensive string manipulation functions. These are **compiler intrinsics** - the compiler emits optimized JavaScript directly instead of calling Sigil functions.

### charAt

Get character at index.

```sigil decl §string
λcharAt(idx:Int,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.charAt(
  0,
  "hello"
)="h" and §string.charAt(
  4,
  "hello"
)="o"
```

**Codegen:** `s.charAt(idx)`

### substring

Get substring from start to end index.

```sigil decl §string
λsubstring(end:Int,s:String,start:Int)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.substring(
  11,
  "hello world",
  6
)="world" and §string.substring(
  3,
  "hello",
  0
)="hel"
```

**Codegen:** `s.substring(start, end)`

### take

Take first n characters.

```sigil decl §string
λtake(n:Int,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.take(
  3,
  "hello"
)="hel" and §string.take(
  5,
  "hi"
)="hi"
```

**Implementation:** `substring(n, s, 0)` (in Sigil)

### drop

Drop first n characters.

```sigil decl §string
λdrop(n:Int,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.drop(
  2,
  "hello"
)="llo" and §string.drop(
  5,
  "hi"
)=""
```

**Implementation:** `substring(#s, s, n)` (in Sigil, uses `#` operator)

### lines

Split a string on newline characters.

```sigil decl §string
λlines(s:String)=>[String]
```

**Examples:**
```sigil program
λmain()=>Bool=§string.lines("a
b
c")=[
  "a",
  "b",
  "c"
] and §string.lines("hello")=["hello"]
```

**Implementation:** `split("
", s)` (in Sigil)

### toUpper

Convert to uppercase.

```sigil decl §string
λtoUpper(s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.toUpper("hello")="HELLO"
```

**Codegen:** `s.toUpperCase()`

### toLower

Convert to lowercase.

```sigil decl §string
λtoLower(s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.toLower("WORLD")="world"
```

**Codegen:** `s.toLowerCase()`

### trim

Remove leading and trailing whitespace.

```sigil decl §string
λtrim(s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.trim("  hello  ")="hello" and §string.trim("
\ttest
")="test"
```

**Codegen:** `s.trim()`

### trimStartChars

Remove any leading characters that appear in `chars`.

```sigil decl §string
λtrimStartChars(chars:String,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.trimStartChars(
  "/",
  "///docs"
)="docs" and §string.trimStartChars(
  "/.",
  "../docs"
)="docs"
```

**Codegen:** edge trim using the characters listed in `chars`

### trimEndChars

Remove any trailing characters that appear in `chars`.

```sigil decl §string
λtrimEndChars(chars:String,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.trimEndChars(
  "/",
  "https://sigil.dev///"
)="https://sigil.dev" and §string.trimEndChars(
  "/.",
  "docs/..."
)="docs"
```

**Codegen:** edge trim using the characters listed in `chars`

### indexOf

Find index of first occurrence (returns -1 if not found).

```sigil decl §string
λindexOf(s:String,search:String)=>Int
```

**Examples:**
```sigil program
λmain()=>Bool=§string.indexOf(
  "hello world",
  "world"
)=6 and §string.indexOf(
  "hello",
  "xyz"
)=-1
```

**Codegen:** `s.indexOf(search)`

### contains

Check whether `search` appears anywhere within `s`.

```sigil decl §string
λcontains(s:String,search:String)=>Bool
```

**Examples:**
```sigil program
λmain()=>Bool=§string.contains(
  "hello world",
  "world"
)
  and ¬§string.contains(
    "hello",
    "xyz"
  )
  and §string.contains(
    "hello",
    ""
  )
```

**Codegen:** `s.includes(search)`

### split

Split string by delimiter.

```sigil decl §string
λsplit(delimiter:String,s:String)=>[String]
```

**Examples:**
```sigil program
λmain()=>Bool=§string.split(
  ",",
  "a,b,c"
)=[
  "a",
  "b",
  "c"
] and §string.split(
  "
",
  "line1
line2"
)=[
  "line1",
  "line2"
]
```

**Codegen:** `s.split(delimiter)`

### replaceAll

Replace all occurrences of pattern with replacement.

```sigil decl §string
λreplaceAll(pattern:String,replacement:String,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.replaceAll(
  "hello",
  "hi",
  "hello hello"
)="hi hi"
```

**Codegen:** `s.replaceAll(pattern, replacement)`

### repeat

Repeat a string `count` times.

```sigil decl §string
λrepeat(count:Int,s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.repeat(
  3,
  "ab"
)="ababab" and §string.repeat(
  0,
  "ab"
)=""
```

**Implementation:** recursive concatenation in Sigil

### reverse

Reverse a string.

```sigil decl §string
λreverse(s:String)=>String
```

**Examples:**
```sigil program
λmain()=>Bool=§string.reverse("stressed")="desserts" and §string.reverse("abc")="cba"
```

**Codegen:** `s.split("").reverse().join("")`

## Current String Surface

`§string` currently exposes:

- `charAt`
- `contains`
- `drop`
- `endsWith`
- `indexOf`
- `intToString`
- `isDigit`
- `join`
- `lines`
- `replaceAll`
- `repeat`
- `reverse`
- `split`
- `startsWith`
- `substring`
- `take`
- `toLower`
- `toUpper`
- `trim`
- `trimEndChars`
- `trimStartChars`
- `unlines`

Design notes:

- use `#s=0` instead of a dedicated `isEmpty`
- use `§string.trim(s)=""` instead of a dedicated whitespace predicate
- use `§string.contains(s,search)` for containment checks

## Float Arithmetic Surface

`§float` provides IEEE 754 double-precision math via JavaScript's `Math` object:

- `abs` — absolute value
- `ceil` — smallest integer ≥ x (returns `Int`)
- `cos` — cosine (radians)
- `exp` — e^x
- `floor` — largest integer ≤ x (returns `Int`)
- `isFinite` — true if x is finite (not ±Infinity, not NaN)
- `isNaN` — true if x is NaN
- `log` — natural logarithm
- `max` — larger of two floats
- `min` — smaller of two floats
- `pow` — base raised to exponent
- `round` — nearest integer, ties round up (returns `Int`)
- `sin` — sine (radians)
- `sqrt` — square root
- `tan` — tangent (radians)
- `toFloat` — convert `Int` to `Float` (exact)
- `toInt` — truncate `Float` toward zero (returns `Int`)

Functions that can produce `NaN` or `±Infinity` (e.g. `sqrt(-1.0)`, `log(0.0)`) return those values as valid `Float`; use `isNaN` and `isFinite` to guard at boundaries.

```sigil program
λmain()=>Bool=§float.floor(3.7)=3
  and §float.ceil(3.2)=4
  and §float.round(2.5)=3
  and §float.isNaN(§float.sqrt(-1.0))
```

## Crypto Surface

`§crypto` provides deterministic hashing and binary-to-text encoding backed by Node.js's `node:crypto` module and `Buffer`:

- `sha256` — SHA-256 hash of a UTF-8 string, hex-encoded
- `hmacSha256` — HMAC-SHA-256 with the given key, hex-encoded
- `base64Encode` — encode UTF-8 string to base64
- `base64Decode` — decode base64 to UTF-8 string (`Err` on invalid input)
- `hexEncode` — encode UTF-8 string to lowercase hex
- `hexDecode` — decode hex to UTF-8 string (`Err` on odd-length or invalid input)

All functions are pure (deterministic, no effect annotation).

```sigil program
λmain()=>Bool match §crypto.base64Decode(§crypto.base64Encode("hello")){
  Ok(s)=>s="hello"|
  Err(_)=>false
}
```

## Current Numeric Surface

`§numeric` currently exposes:

- `abs`
- `clamp`
- `divisible`
- `divmod`
- `gcd`
- `inRange`
- `isEven`
- `isNegative`
- `isNonNegative`
- `isOdd`
- `isPositive`
- `isPrime`
- `lcm`
- `max`
- `min`
- `mod`
- `pow`
- `range`
- `sign`

Examples:

```sigil program
λmain()=>Bool=§numeric.abs(-5)=5
  and §numeric.isEven(4)
  and §numeric.isPrime(17)
  and §numeric.range(
    2,
    5
  )=[
    2,
    3,
    4,
    5
  ]
```

## Core Prelude

`ConcurrentOutcome[T,E]`, `Option[T]`, `Result[T,E]`, `Aborted`, `Failure`,
`Success`, `Some`, `None`, `Ok`, and `Err` are part of the implicit
`¶prelude`. They do not require qualification.

Current canonical type forms:

```sigil module
t ConcurrentOutcome[T,E]=Aborted()|Failure(E)|Success(T)

t Option[T]=Some(T)|None()

t Result[T,E]=Ok(T)|Err(E)
```

Typical usage:

```sigil module
λgetOrDefault(default:Int,opt:Option[Int])=>Int match opt{
  Some(value)=>value|
  None()=>default
}

λprocessResult(res:Result[
  String,
  String
])=>String match res{
  Ok(value)=>"Success: "++value|
  Err(msg)=>"Error: "++msg
}
```

## Core Map

`¶map` is the canonical helper surface for `{K↦V}` values.

Canonical type and literal forms:

```sigil module
t Headers={String↦String}

c empty=(({↦}:{String↦String}):{String↦String})

c filled=({"content-type"↦"text/plain"}:{String↦String})
```

Canonical helper surface:

```sigil module
```

## Stability Note

This document describes the current shipped stdlib surface. Placeholder future APIs and older snake_case names are intentionally omitted here. When the surface changes, update the checked declarations and examples in this file instead of keeping speculative or legacy aliases around.
