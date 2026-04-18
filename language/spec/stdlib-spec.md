# Sigil Standard Library Specification

Version: 1.0.0
Last Updated: 2026-03-07

## Overview

The Sigil standard library provides essential types and functions that are automatically available in every Sigil program. The design philosophy emphasizes:

1. **Minimal but complete** - Only include truly universal functionality
2. **Functional-first** - Pure functions, immutability by default
3. **Type-safe** - Leverage strong type system
4. **Composable** - Functions that work well together
5. **Zero-cost abstractions** - Compile to efficient JavaScript

## Implicit Prelude and Rooted Modules

The prelude is available in every Sigil module without qualification. Other
modules are reached through rooted references such as `§list`, `•topology`,
`†runtime`, `※check::log`, and `☴router`.

## Core Types

### ConcurrentOutcome[T,E]

Implicit core prelude sum type:

```sigil decl ¶prelude
t ConcurrentOutcome[T,E]=Aborted()|Failure(E)|Success(T)
```

- `Aborted[T,E]()=>ConcurrentOutcome[T,E]`
- `Failure[T,E](error:E)=>ConcurrentOutcome[T,E]`
- `Success[T,E](value:T)=>ConcurrentOutcome[T,E]`

### Option[T]

Represents an optional value - Sigil's null-safe alternative.

```sigil module
t Option[T]=Some(T)|None()
```

**Constructors:**
- `Some[T](value:T)=>Option[T]` - Wraps a value
- `None[T]()=>Option[T]` - Represents absence

**Functions:**

```text
mapOption(fn,opt)
bindOption(fn,opt)
unwrapOr(fallback,opt)
isSome(opt)
isNone(opt)
```

### Result[T,E]

Represents a computation that may fail - Sigil's exception-free error handling.

```sigil module
t Result[T,E]=Ok(T)|Err(E)
```

**Constructors:**
- `Ok[T,E](value:T)=>Result[T,E]` - Success case
- `Err[T,E](error:E)=>Result[T,E]` - Error case

**Functions:**

```text
mapResult(fn,res)
bindResult(fn,res)
unwrapOrResult(fallback,res)
isOk(res)
isErr(res)
```

## List Operations

### Implemented `§list` Functions

```sigil decl §list
λall[T](pred:λ(T)=>Bool,xs:[T])=>Bool
λany[T](pred:λ(T)=>Bool,xs:[T])=>Bool
λcontains[T](item:T,xs:[T])=>Bool
λcount[T](item:T,xs:[T])=>Int
λcountIf[T](pred:λ(T)=>Bool,xs:[T])=>Int
λdrop[T](n:Int,xs:[T])=>[T]
λfind[T](pred:λ(T)=>Bool,xs:[T])=>Option[T]
λflatMap[T,U](fn:λ(T)=>[U],xs:[T])=>[U]
λfold[T,U](acc:U,fn:λ(U,T)=>U,xs:[T])=>U
λinBounds[T](idx:Int,xs:[T])=>Bool
λlast[T](xs:[T])=>Option[T]
λmax(xs:[Int])=>Option[Int]
λmin(xs:[Int])=>Option[Int]
λnth[T](idx:Int,xs:[T])=>Option[T]
λproduct(xs:[Int])=>Int
λremoveFirst[T](item:T,xs:[T])=>[T]
λreverse[T](xs:[T])=>[T]
λsortedAsc(xs:[Int])=>Bool
λsortedDesc(xs:[Int])=>Bool
λsum(xs:[Int])=>Int
λtake[T](n:Int,xs:[T])=>[T]
```

Safe element access uses `Option[T]`:
- `last([])=>None()`
- `find(pred,[])=>None()`
- `max([])=>None()`
- `min([])=>None()`
- `nth(-1,xs)=>None()`
- `nth(idx,xs)=>None()` when out of bounds

### Canonical list-processing restrictions

Sigil treats the list-processing surface as canonical:

- use `§list.all` for universal checks
- use `§list.any` for existential checks
- use `§list.countIf` for predicate counting
- use `map` for projection
- use `filter` for filtering
- use `§list.find` for first-match search
- use `§list.flatMap` for flattening projection
- use `reduce ... from ...` or `§list.fold` for reduction
- use `§list.reverse` for reversal

The validator rejects exact recursive clones of `all`, `any`, `map`, `filter`,
`find`, `flatMap`, `fold`, and `reverse`, rejects `#(xs filter pred)` in favor of
`§list.countIf`, and rejects recursive result-building of the form
`self(rest)⧺rhs`. These are narrow AST-shape rules, not a general complexity
prover.

Outside `language/stdlib/`, the validator also rejects exact top-level wrappers
whose body is already a canonical helper surface, such as `§list.sum(xs)`,
`§numeric.max(a,b)`, `§string.trim(s)`, `xs map fn`, `xs filter pred`, or
`xs reduce fn from init`. Sigil keeps one canonical helper surface instead of
supporting thin local aliases for the same operation.

### Implemented `§numeric` Helpers

```sigil decl §numeric
t DivMod={quotient:Int,remainder:Int}
λabs(x:Int)=>Int
λclamp(hi:Int,lo:Int,x:Int)=>Int
λdivisible(d:Int,n:Int)=>Bool
λdivmod(a:Int,b:Int)=>DivMod
λgcd(a:Int,b:Int)=>Int
λinRange(max:Int,min:Int,x:Int)=>Bool
λisEven(x:Int)=>Bool
λisNegative(x:Int)=>Bool
λisNonNegative(x:Int)=>Bool
λisOdd(x:Int)=>Bool
λisPositive(x:Int)=>Bool
λisPrime(n:Int)=>Bool
λlcm(a:Int,b:Int)=>Int
λmax(a:Int,b:Int)=>Int
λmin(a:Int,b:Int)=>Int
λmod(a:Int,b:Int)=>Int
λpow(base:Int,exp:Int)=>Int
λrange(start:Int,stop:Int)=>[Int]
λsign(x:Int)=>Int
```

### Implemented `§random` Functions

```sigil decl §random
λintBetween(max:Int,min:Int)=>!Random Int
λpick[T](items:[T])=>!Random Option[T]
λshuffle[T](items:[T])=>!Random [T]
```

Semantics:
- `intBetween` is inclusive and order-insensitive over its two bounds
- `pick([])` returns `None()`
- `shuffle` returns a full permutation of the input list
- runtime behavior comes from the active world's `random` entry

## Feature Flags

`§featureFlags` is the canonical stdlib surface for evaluating first-class
`featureFlag` declarations.

Current types:

```sigil decl §featureFlags
t Config[T,C]={key:Option[λ(C)=>Option[String]],rules:[Rule[T,C]]}
t Entry[C]
t Flag[T]={createdAt:String,default:T,id:String}
t RolloutPlan[T]={percentage:Int,variants:[WeightedValue[T]]}
t Rule[T,C]={action:RuleAction[T],predicate:λ(C)=>Bool}
t RuleAction[T]=Rollout(RolloutPlan[T])|Value(T)
t Set[C]=[Entry[C]]
t WeightedValue[T]={value:T,weight:Int}
```

Current functions:

```sigil decl §featureFlags
λentry[C,T](config:Config[T,C],flag:Flag[T])=>Entry[C]
λget[C,T](context:C,flag:Flag[T],set:Set[C])=>T
```

Current `§featureFlags.get` semantics:

1. resolve the configured key function, if any
2. otherwise evaluate rules in order and stop at the first matching predicate
3. `Value(v)` returns `v`
4. `Rollout(r)` requires a resolved key and hashes `(flag.id,key)`
   deterministically into the weighted rollout variants, gated by `percentage`
5. if no rule matches, return `flag.default`

## String Operations

```sigil decl §string
λcharAt(idx:Int,s:String)=>String
λcontains(s:String,search:String)=>Bool
λdrop(n:Int,s:String)=>String
λendsWith(s:String,suffix:String)=>Bool
λindexOf(s:String,search:String)=>Int
λintToString(n:Int)=>String
λisDigit(s:String)=>Bool
λjoin(separator:String,strings:[String])=>String
λlines(s:String)=>[String]
λreplaceAll(pattern:String,replacement:String,s:String)=>String
λrepeat(count:Int,s:String)=>String
λreverse(s:String)=>String
λsplit(delimiter:String,s:String)=>[String]
λstartsWith(prefix:String,s:String)=>Bool
λsubstring(end:Int,s:String,start:Int)=>String
λtake(n:Int,s:String)=>String
λtoLower(s:String)=>String
λtoUpper(s:String)=>String
λtrimEndChars(chars:String,s:String)=>String
λtrimStartChars(chars:String,s:String)=>String
λtrim(s:String)=>String
λunlines(lines:[String])=>String
```

## File and Process Operations

### Implemented `§file` Functions

```sigil decl §file
λappendText(content:String,path:String)=>!Fs Unit
λappendTextAt(content:String,path:String,handle:§topology.FsRoot)=>!Fs Unit
λexists(path:String)=>!Fs Bool
λexistsAt(path:String,handle:§topology.FsRoot)=>!Fs Bool
λlistDir(path:String)=>!Fs [String]
λlistDirAt(path:String,handle:§topology.FsRoot)=>!Fs [String]
λmakeDir(path:String)=>!Fs Unit
λmakeDirAt(path:String,handle:§topology.FsRoot)=>!Fs Unit
λmakeDirs(path:String)=>!Fs Unit
λmakeDirsAt(path:String,handle:§topology.FsRoot)=>!Fs Unit
λmakeTempDir(prefix:String)=>!Fs String
λmakeTempDirAt(prefix:String,handle:§topology.FsRoot)=>!Fs String
λreadText(path:String)=>!Fs String
λreadTextAt(path:String,handle:§topology.FsRoot)=>!Fs String
λremove(path:String)=>!Fs Unit
λremoveAt(path:String,handle:§topology.FsRoot)=>!Fs Unit
λremoveTree(path:String)=>!Fs Unit
λremoveTreeAt(path:String,handle:§topology.FsRoot)=>!Fs Unit
λwriteText(content:String,path:String)=>!Fs Unit
λwriteTextAt(content:String,path:String,handle:§topology.FsRoot)=>!Fs Unit
```

`makeTempDir(prefix)` creates a fresh temp directory and returns its absolute
path. Cleanup remains explicit through `removeTree`.

The `*At` variants are the named-boundary surface for topology-aware projects.

### Implemented `§cli` Types and Functions

```sigil decl §cli
t Program[T]
t RootCommand[T]
t Command[T]
t Arg[A]

λprogram[T](description:String,name:String,root:Option[RootCommand[T]],subcommands:[Command[T]])=>Program[T]

λrun[T](argv:[String],program:Program[T])=>!Log!Process T

λroot0[T](description:String,result:T)=>RootCommand[T]
λroot1[A,T](arg1:Arg[A],build:λ(A)=>T,description:String)=>RootCommand[T]
λroot2[A,B,T](arg1:Arg[A],arg2:Arg[B],build:λ(A,B)=>T,description:String)=>RootCommand[T]
λroot3[A,B,C,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],build:λ(A,B,C)=>T,description:String)=>RootCommand[T]
λroot4[A,B,C,D,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],arg4:Arg[D],build:λ(A,B,C,D)=>T,description:String)=>RootCommand[T]
λroot5[A,B,C,D,E,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],arg4:Arg[D],arg5:Arg[E],build:λ(A,B,C,D,E)=>T,description:String)=>RootCommand[T]
λroot6[A,B,C,D,E,F,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],arg4:Arg[D],arg5:Arg[E],arg6:Arg[F],build:λ(A,B,C,D,E,F)=>T,description:String)=>RootCommand[T]

λcommand0[T](description:String,name:String,result:T)=>Command[T]
λcommand1[A,T](arg1:Arg[A],build:λ(A)=>T,description:String,name:String)=>Command[T]
λcommand2[A,B,T](arg1:Arg[A],arg2:Arg[B],build:λ(A,B)=>T,description:String,name:String)=>Command[T]
λcommand3[A,B,C,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],build:λ(A,B,C)=>T,description:String,name:String)=>Command[T]
λcommand4[A,B,C,D,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],arg4:Arg[D],build:λ(A,B,C,D)=>T,description:String,name:String)=>Command[T]
λcommand5[A,B,C,D,E,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],arg4:Arg[D],arg5:Arg[E],build:λ(A,B,C,D,E)=>T,description:String,name:String)=>Command[T]
λcommand6[A,B,C,D,E,F,T](arg1:Arg[A],arg2:Arg[B],arg3:Arg[C],arg4:Arg[D],arg5:Arg[E],arg6:Arg[F],build:λ(A,B,C,D,E,F)=>T,description:String,name:String)=>Command[T]

λflag(description:String,long:String,short:Option[String])=>Arg[Bool]
λoption(description:String,long:String,short:Option[String],valueName:String)=>Arg[Option[String]]
λrequiredOption(description:String,long:String,short:Option[String],valueName:String)=>Arg[String]
λmanyOption(description:String,long:String,short:Option[String],valueName:String)=>Arg[[String]]
λpositional(description:String,name:String)=>Arg[String]
λoptionalPositional(description:String,name:String)=>Arg[Option[String]]
λmanyPositionals(description:String,name:String)=>Arg[[String]]
```

`§cli` is the canonical typed CLI layer above `§process.argv()`.

CLI rules:
- `§cli.run` owns help and parse-failure output
- help exits `0`
- parse failure exits `2`
- v1 supports one subcommand layer only
- option values stay string-based in v1
- `§process.argv()` remains the only raw argv source

### Implemented `§process` Types and Functions

```sigil decl §process
t Command={argv:[String],cwd:Option[String],env:{String↦String}}
t RunningProcess={pid:Int}
t ProcessResult={code:Int,stderr:String,stdout:String}
t ProcessFailure={code:Int,stderr:String,stdout:String}

λcommand(argv:[String])=>Command
λexit(code:Int)=>!Process Never
λrun(command:Command)=>!Process ProcessResult
λrunAt(command:Command,handle:§topology.ProcessHandle)=>!Process ProcessResult
λrunChecked(command:Command)=>!Process Result[ProcessResult,ProcessFailure]
λrunJson(command:Command)=>!Process Result[§json.JsonValue,ProcessFailure]
λstart(command:Command)=>!Process Owned[RunningProcess]
λstartAt(command:Command,handle:§topology.ProcessHandle)=>!Process Owned[RunningProcess]
λwithCwd(command:Command,cwd:String)=>Command
λwithEnv(command:Command,env:{String↦String})=>Command
λwait(process:RunningProcess)=>!Process ProcessResult
λkill(process:RunningProcess)=>!Process Unit
```

Process rules:
- command execution is argv-based only
- `withEnv` overlays explicit variables on top of the inherited environment
- non-zero exit codes are reported in `ProcessResult.code`
- `run` captures stdout and stderr in memory
- `runChecked` converts non-zero exit into `Err(ProcessFailure)`
- `runJson` requires zero exit and then parses stdout as JSON
- `start` and `startAt` return owned process handles
- `runAt` and `startAt` are the named-boundary variants for topology-aware projects
- `kill` is a normal termination request, not a timeout/escalation protocol
- `exit` terminates the current process and has result type `Never`

### Implemented `§fsWatch` Types and Functions

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
- `watch` and `watchAt` return owned watch handles
- `watchAt` is the topology-aware named-boundary variant and requires `§topology.FsRoot`
- rename detection is not modeled separately in v1

### Implemented `§pty` Types and Functions

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
- PTY sessions expose one combined terminal stream rather than split stdout/stderr
- `events` yields `Output(text)` chunks and then one `Exit(code)` when the session terminates normally
- `wait` resolves to the final exit code for that session
- `close` is a normal session shutdown request
- `spawn` and `spawnAt` return owned session handles
- `spawnManaged` and `spawnManagedAt` return storable runtime-managed session refs
- `eventsManaged` returns an owned subscription stream for one managed session ref
- `closeManaged` is idempotent
- `spawnAt` is the topology-aware named-boundary variant and requires `§topology.PtyHandle`
- `spawnManagedAt` is the topology-aware managed-ref variant and requires `§topology.PtyHandle`

### Implemented `§stream` Types and Functions

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
- `hub` and `subscribe` return owned handles
- `publish` fanouts to current subscribers in send order
- generic stream failure is not modeled in `§stream`; producer APIs own their error events
- `§stream` intentionally omits combinator-style operator families in v1

### Implemented `§websocket` Types and Functions

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
- `connections` yields accepted clients scoped to one exact `§topology.WebSocketHandle`
- `messages` yields text frames for one client
- `listen`, `connections`, and `messages` return owned handles
- `send` writes one text frame to one client
- `close` closes one client connection
- v1 is server-only and does not expose binary frames, subprotocol negotiation, or a broadcast helper

### Implemented `§terminal` Types and Functions

```sigil decl §terminal
t Key=Escape()|Text(String)

λclearScreen()=>!Terminal Unit
λdisableRawMode()=>!Terminal Unit
λenableRawMode()=>!Terminal Unit
λhideCursor()=>!Terminal Unit
λreadKey()=>!Terminal Key
λshowCursor()=>!Terminal Unit
λwrite(text:String)=>!Terminal Unit
```

Terminal rules:
- terminal interaction is raw-key oriented rather than line-oriented
- `readKey` returns canonical `Key` values
- `Escape()` represents the escape key and escape sequences
- `Text(String)` carries normalized plain-text key input
- interactive programs should restore cursor visibility and raw-mode state before exit

### Implemented `§regex` Types and Functions

```sigil decl §regex
t Regex={flags:String,pattern:String}
t RegexError={message:String}
t RegexMatch={captures:[String],end:Int,full:String,start:Int}

λcompile(flags:String,pattern:String)=>Result[Regex,RegexError]
λfind(input:String,regex:Regex)=>Option[RegexMatch]
λfindAll(input:String,regex:Regex)=>[RegexMatch]
λisMatch(input:String,regex:Regex)=>Bool
```

Regex rules:
- semantics follow JavaScript `RegExp`
- `compile` validates both flags and pattern before returning `Ok`
- `find` returns the first match only
- `findAll` returns all non-overlapping matches; adds the `g` flag internally
- unmatched capture groups are returned as empty strings in `captures`

### Implemented `§float` Types and Functions

```sigil decl §float
λabs(x:Float)=>Float
λceil(x:Float)=>Int
λcos(x:Float)=>Float
λexp(x:Float)=>Float
λfloor(x:Float)=>Int
λisFinite(x:Float)=>Bool
λisNaN(x:Float)=>Bool
λlog(x:Float)=>Float
λmax(a:Float,b:Float)=>Float
λmin(a:Float,b:Float)=>Float
λpow(base:Float,exp:Float)=>Float
λround(x:Float)=>Int
λsin(x:Float)=>Float
λsqrt(x:Float)=>Float
λtan(x:Float)=>Float
λtoFloat(x:Int)=>Float
λtoInt(x:Float)=>Int
```

Float rules:
- all functions delegate to `Math.*` or `Number.*` in the JS runtime
- `ceil`, `floor`, `round`, `toInt` return `Int` (not `Float`)
- `toInt` truncates toward zero (equivalent to `Math.trunc`)
- `log` is the natural logarithm
- functions producing `NaN` or `±Infinity` do so silently; use `isNaN` / `isFinite` to guard

### Implemented `§crypto` Types and Functions

```sigil decl §crypto
t CryptoError={message:String}

λbase64Decode(input:String)=>Result[String,CryptoError]
λbase64Encode(input:String)=>String
λhexDecode(input:String)=>Result[String,CryptoError]
λhexEncode(input:String)=>String
λhmacSha256(key:String,message:String)=>String
λsha256(input:String)=>String
```

Crypto rules:
- all functions are pure (no effect annotation); all inputs are treated as UTF-8
- `sha256` and `hmacSha256` return lowercase hex strings
- `base64Decode` and `hexDecode` return `Err` on invalid input; `hexDecode` additionally errors on odd-length input
- backed by `node:crypto` (`createHash`, `createHmac`) and `Buffer`

### Implemented `§time` Additions

```sigil decl §time
λsleepMs(ms:Int)=>!Timer Unit
```

`sleepMs` is the canonical delay primitive for retry loops and harness
orchestration.

### Implemented `§timer` Types and Functions

```sigil decl §timer
λafterMs(ms:Int)=>!Timer Owned[§stream.Source[Unit]]
λeveryMs(ms:Int)=>!Timer Owned[§stream.Source[Unit]]
```

Semantics:
- `afterMs` yields one `()` tick and then finishes
- `everyMs` yields repeated `()` ticks until the source is closed
- both functions return owned stream sources

### Implemented `§task` Types and Functions

```sigil decl §task
t Task[T]={id:Int}
t TaskResult[T]=Cancelled()|Failed(String)|Succeeded(T)

λcancel[T](task:Task[T])=>!Task Unit
λspawn[T](work:λ()=>T)=>!Task Owned[Task[T]]
λwait[T](task:Task[T])=>!Task TaskResult[T]
```

Semantics:
- `spawn` returns an owned task handle
- `cancel` requests cancellation
- `wait` resolves to `Succeeded(value)`, `Cancelled()`, or `Failed(message)`

## Map Operations

Maps are a core collection concept, and helper functions live in `¶map`.

```sigil decl ¶map
t Entry[K,V]={key:K,value:V}

λempty[K,V]()=>{K↦V}
λentries[K,V](map:{K↦V})=>[Entry[K,V]]
λfilter[K,V](map:{K↦V},pred:λ(K,V)=>Bool)=>{K↦V}
λfold[K,V,U](fn:λ(U,K,V)=>U,init:U,map:{K↦V})=>U
λfromList[K,V](entries:[Entry[K,V]])=>{K↦V}
λget[K,V](key:K,map:{K↦V})=>Option[V]
λhas[K,V](key:K,map:{K↦V})=>Bool
λinsert[K,V](key:K,map:{K↦V},value:V)=>{K↦V}
λkeys[K,V](map:{K↦V})=>[K]
λmapValues[K,V,U](fn:λ(V)=>U,map:{K↦V})=>{K↦U}
λmerge[K,V](left:{K↦V},right:{K↦V})=>{K↦V}
λremove[K,V](key:K,map:{K↦V})=>{K↦V}
λsingleton[K,V](key:K,value:V)=>{K↦V}
λsize[K,V](map:{K↦V})=>Int
λvalues[K,V](map:{K↦V})=>[V]
```

## JSON Operations

```sigil decl §json
t JsonError={message:String}
t JsonValue=JsonArray([JsonValue])|JsonBool(Bool)|JsonNull|JsonNumber(Float)|JsonObject({String↦JsonValue})|JsonString(String)

λparse(input:String)=>Result[JsonValue,JsonError]
λstringify(value:JsonValue)=>String
λgetField(key:String,obj:{String↦JsonValue})=>Option[JsonValue]
λgetIndex(arr:[JsonValue],idx:Int)=>Option[JsonValue]
λasArray(value:JsonValue)=>Option[[JsonValue]]
λasBool(value:JsonValue)=>Option[Bool]
λasNumber(value:JsonValue)=>Option[Float]
λasObject(value:JsonValue)=>Option[{String↦JsonValue}]
λasString(value:JsonValue)=>Option[String]
λisNull(value:JsonValue)=>Bool
```

Notes:
- `parse` is exception-safe and returns `Err({message})` for invalid JSON.
- `stringify` is canonical JSON output for the provided `JsonValue`.

## Decode Operations

`§decode` is the canonical boundary layer from raw `JsonValue` to trusted
internal Sigil values.

```sigil decl §decode
t DecodeError={message:String,path:[String]}
t Decoder[T]=λ(JsonValue)=>Result[T,DecodeError]

λrun[T](decoder:Decoder[T],value:JsonValue)=>Result[T,DecodeError]
λparse[T](decoder:Decoder[T],input:String)=>Result[T,DecodeError]
λsucceed[T](value:T)=>Decoder[T]
λfail[T](message:String)=>Decoder[T]
λmap[T,U](decoder:Decoder[T],fn:λ(T)=>U)=>Decoder[U]
λbind[T,U](decoder:Decoder[T],fn:λ(T)=>Decoder[U])=>Decoder[U]

λbool(value:JsonValue)=>Result[Bool,DecodeError]
λfloat(value:JsonValue)=>Result[Float,DecodeError]
λint(value:JsonValue)=>Result[Int,DecodeError]
λstring(value:JsonValue)=>Result[String,DecodeError]

λlist[T](decoder:Decoder[T])=>Decoder[[T]]
λdict[T](decoder:Decoder[T])=>Decoder[{String↦T}]
λfield[T](decoder:Decoder[T],key:String)=>Decoder[T]
λoptionalField[T](decoder:Decoder[T],key:String)=>Decoder[Option[T]]
```

Notes:
- `§json` owns raw parsing and inspection.
- `§decode` owns conversion into trusted internal types.
- `DecodeError.path` records the nested field/index path of the failure.
- If a field may be absent, keep the record exact and use `Option[T]` for that field.
- Sigil does not use open records or partial records for this boundary story.

## Time Operations

```sigil decl §time
t Instant={epochMillis:Int}
t TimeError={message:String}

λparseIso(input:String)=>Result[Instant,TimeError]
λformatIso(instant:Instant)=>String
λnow()=>!Clock Instant
λfromEpochMillis(millis:Int)=>Instant
λtoEpochMillis(instant:Instant)=>Int
λcompare(left:Instant,right:Instant)=>Int
λisBefore(left:Instant,right:Instant)=>Bool
λisAfter(left:Instant,right:Instant)=>Bool
```

Notes:
- `parseIso` is strict ISO-8601 only.
- Non-ISO text must be normalized before calling `parseIso`.

## Math Operations

The numeric helper surface is owned by `§numeric`; there is no separate
math module today.

## Logging Operations

```sigil decl §io
λdebug(msg:String)=>!Log Unit
λeprintln(msg:String)=>!Log Unit
λprint(msg:String)=>!Log Unit
λprintln(msg:String)=>!Log Unit
λwarn(msg:String)=>!Log Unit
```

```sigil decl §log
λwrite(message:String,sink:§topology.LogSink)=>!Log Unit
```

`§log.write` is the named-boundary logging surface used by labelled
boundary rules.

## Module System

### Import Syntax

```sigil module
```

### Export Visibility

File extension determines visibility:

**`.lib.sigil` files** (libraries):
- All top-level declarations are automatically visible to other modules
- No `export` keyword needed or allowed

**`.sigil` files** (executables):
- Export nothing directly
- Have `main()` function

No import declarations, no aliasing, no export lists.

## Standard Library Modules

### core/prelude

Implicitly available. Contains the foundational vocabulary types:
- `Option[T]`
- `Result[T,E]`
- `Some`
- `None`
- `Ok`
- `Err`

### §file

UTF-8 filesystem helpers:
- `appendText`
- `exists`
- `listDir`
- `makeDir`
- `makeDirs`
- `makeTempDir`
- `readText`
- `remove`
- `removeTree`
- `writeText`

### §path

Filesystem path helpers:
- `basename`
- `dirname`
- `extname`
- `join`
- `normalize`
- `relative`

### §io

Console and process I/O only (`print`, `println`, `eprintln`, `warn`, `debug`)

### ¶map

Dynamic keyed collection helpers over `{K↦V}` values.

### §numeric

Integer helpers (`abs`, `divmod`, `gcd`, `lcm`, `max`, `min`, `mod`,
predicates like `isEven`, and ranges).

### §json

Typed JSON parsing and serialization (`JsonValue`, `parse`, `stringify`)

```sigil decl §json
λparse(input:String)=>Result[JsonValue,JsonError]
λstringify(value:JsonValue)=>String
```

### §decode

Canonical JSON-to-domain decoding (`Decoder[T]`, `DecodeError`, `run`, `parse`)

```sigil decl §decode
λrun[T](decoder:Decoder[T],value:JsonValue)=>Result[T,DecodeError]
λparse[T](decoder:Decoder[T],input:String)=>Result[T,DecodeError]
```

### §time

Time and instant handling (`Instant`, strict ISO parsing, clock access)

```sigil decl §time
λparseIso(input:String)=>Result[Instant,TimeError]
λformatIso(instant:Instant)=>String
λnow()=>!Clock Instant
```

### §topology

Canonical declaration layer for external HTTP and TCP runtime dependencies.

```sigil decl §topology
t Environment=Environment(String)
t HttpServiceDependency=HttpServiceDependency(String)
t TcpServiceDependency=TcpServiceDependency(String)

λenvironment(name:String)=>Environment
λhttpService(name:String)=>HttpServiceDependency
λtcpService(name:String)=>TcpServiceDependency
```

### §config

Low-level helper layer for topology-backed environment config data.

Canonical project environment files now export `world` values built through
`†runtime`, `†http`, and `†tcp`. `§config` remains
available inside config modules for binding-shaped helper values, but it is no
longer the exported environment ABI.

```sigil decl §config
t BindingValue=EnvVar(String)|Literal(String)
t Bindings={httpBindings:[HttpBinding],tcpBindings:[TcpBinding]}
t HttpBinding={baseUrl:BindingValue,dependencyName:String}
t PortBindingValue=EnvVarPort(String)|LiteralPort(Int)
t TcpBinding={dependencyName:String,host:BindingValue,port:PortBindingValue}

λbindHttp(baseUrl:String,dependency:§topology.HttpServiceDependency)=>HttpBinding
λbindHttpEnv(dependency:§topology.HttpServiceDependency,envVar:String)=>HttpBinding
λbindTcp(dependency:§topology.TcpServiceDependency,host:String,port:Int)=>TcpBinding
λbindTcpEnv(dependency:§topology.TcpServiceDependency,hostEnvVar:String,portEnvVar:String)=>TcpBinding
λbindings(httpBindings:[HttpBinding],tcpBindings:[TcpBinding])=>Bindings
```

### §httpClient

Canonical text-based HTTP client.

```sigil decl §httpClient
t Headers={String↦String}
t HttpError={kind:HttpErrorKind,message:String}
t HttpErrorKind=InvalidJson()|InvalidUrl()|Network()|Timeout()|Topology()
t HttpMethod=Delete()|Get()|Patch()|Post()|Put()
t HttpRequest={body:Option[String],dependency:§topology.HttpServiceDependency,headers:Headers,method:HttpMethod,path:String}
t HttpResponse={body:String,headers:Headers,status:Int,url:String}

λrequest(request:HttpRequest)=>!Http Result[HttpResponse,HttpError]
λget(dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[HttpResponse,HttpError]
λdelete(dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[HttpResponse,HttpError]
λpost(body:String,dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[HttpResponse,HttpError]
λput(body:String,dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[HttpResponse,HttpError]
λpatch(body:String,dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[HttpResponse,HttpError]

λgetJson(dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[JsonValue,HttpError]
λdeleteJson(dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[JsonValue,HttpError]
λpostJson(body:JsonValue,dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[JsonValue,HttpError]
λputJson(body:JsonValue,dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[JsonValue,HttpError]
λpatchJson(body:JsonValue,dependency:§topology.HttpServiceDependency,headers:Headers,path:String)=>!Http Result[JsonValue,HttpError]
λresponseJson(response:HttpResponse)=>Result[JsonValue,HttpError]

λemptyHeaders()=>Headers
λjsonHeaders()=>Headers
λheader(key:String,value:String)=>Headers
λmergeHeaders(left:Headers,right:Headers)=>Headers
```

Semantics:
- any successfully received HTTP response returns `Ok(HttpResponse)`, including `404` and `500`
- invalid URL, transport failure, topology resolution failure, and JSON parse failure return `Err(HttpError)`
- request and response bodies are UTF-8 text in v1

### §httpServer

Canonical request/response HTTP server.

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

λresponse(body:String,contentType:String,status:Int)=>Response
λok(body:String)=>Response
λjson(body:String,status:Int)=>Response
λjsonBody(request:Request)=>Result[§json.JsonValue,HttpBodyError]
λlisten(port:Int)=>!Http Owned[Server]
λlistenWithWebSockets(port:Int,routes:[WebSocketRoute])=>!Http Owned[Server]
λlistenWith(handler:λ(Request)=>Response,port:Int)=>!Http Server
λmatch(method:String,pathPattern:String,request:Request)=>Option[RouteMatch]
λnotFound()=>Response
λnotFoundMsg(path:String)=>Response
λport(server:Server)=>Int
λreply(responder:Responder,response:Response)=>!Http Unit
λrequests(server:Server)=>!Http Owned[§stream.Source[PendingRequest]]
λserverError(message:String)=>Response
λlogRequest(request:Request)=>!Log Unit
λserve(handler:λ(Request)=>Response,port:Int)=>!Http Unit
λwait(server:Server)=>!Http Unit
λwebsocketClose(client:WebSocketClient)=>!Http Unit
λwebsocketConnections(handle:§topology.WebSocketHandle,server:Server)=>!Http Owned[§stream.Source[WebSocketClient]]
λwebsocketMessages(client:WebSocketClient)=>!Http Owned[§stream.Source[String]]
λwebsocketRoute(handle:§topology.WebSocketHandle,path:String)=>WebSocketRoute
λwebsocketSend(client:WebSocketClient,text:String)=>!Http Unit
```

Semantics:
- `listen(port)` returns an owned server handle for request-stream orchestration
- `listenWithWebSockets(port,routes)` returns one owned HTTP server handle that also owns exact-path websocket upgrades on the same bound port
- `requests(server)` returns an owned stream of `PendingRequest` values
- `reply` answers one pending request through its `Responder`
- `listenWith(handler,port)` and `serve(handler,port)` remain available for simple pure-handler programs
- passing `0` as the port asks the OS to choose any free ephemeral port
- `port(server)` returns the actual bound port, including after a `0` bind
- `serve` and `wait` are long-lived once listening succeeds
- `websocketRoute` declares one exact websocket upgrade path for one `§topology.WebSocketHandle`
- `websocketConnections` yields accepted websocket clients for one shared-listener route
- `websocketMessages` yields text frames for one websocket client
- `websocketSend` and `websocketClose` act on one websocket client connected through the shared listener

### §tcpClient

Canonical one-request, one-response TCP client.

```sigil decl §tcpClient
t TcpError={kind:TcpErrorKind,message:String}
t TcpErrorKind=Connection()|InvalidAddress()|Protocol()|Timeout()|Topology()
t TcpRequest={dependency:§topology.TcpServiceDependency,message:String}
t TcpResponse={message:String}

λrequest(request:TcpRequest)=>!Tcp Result[TcpResponse,TcpError]
λsend(dependency:§topology.TcpServiceDependency,message:String)=>!Tcp Result[TcpResponse,TcpError]
```

Semantics:
- requests are UTF-8 text
- the client writes one newline-delimited message and expects one newline-delimited response
- address validation, socket failure, timeout, topology resolution failure, and framing failure return `Err(TcpError)`

### §tcpServer

Canonical one-request, one-response TCP server.

```sigil decl §tcpServer
t Request={host:String,message:String,port:Int}
t Response={message:String}
t Server={port:Int}

λlisten(handler:λ(Request)=>Response,port:Int)=>!Tcp Server
λport(server:Server)=>Int
λresponse(message:String)=>Response
λserve(handler:λ(Request)=>Response,port:Int)=>!Tcp Unit
λwait(server:Server)=>!Tcp Unit
```

Semantics:
- the server reads one UTF-8 line per connection
- the handler returns one UTF-8 line response
- the server closes each connection after the response is written
- `serve(handler,port)` is equivalent to blocking on a started server
- `listen` returns a server handle that can be observed with `port` and awaited with `wait`
- passing `0` as the port asks the OS to choose any free ephemeral port
- `port(server)` returns the actual bound port, including after a `0` bind
- `serve` and `wait` are long-lived once listening succeeds

### Testing

Testing is built into the language with `test` declarations and the `sigil
test` runner. There is no current `§test` module surface.

## Implementation Notes

### JavaScript Compilation

- Lists compile to JavaScript arrays
- Maps compile to JavaScript Map objects
- Strings are JavaScript strings (UTF-16)
- Integers are JavaScript numbers (beware 32-bit limits!)
- Floats are JavaScript numbers (IEEE 754 double)

### Performance Considerations

- List operations are functional (immutable) - use sparingly for large lists
- For performance-critical code, consider using mutable collections explicitly
- String concatenation in loops is O(n²) - prefer §string.join when building from parts

### Effect System

Effects are tracked at type level:
- `!Clock`
- `!Fs`
- `!FsWatch`
- `!Http`
- `!Log`
- `!Process`
- `!Pty`
- `!Random`
- `!Stream`
- `!Tcp`
- `!Terminal`
- `!Timer`
- `!WebSocket`
- Pure functions have no effect annotation

Projects may define reusable multi-effect aliases in `src/effects.lib.sigil`.

## Future Extensions

Planned for future stdlib versions:

- **§concurrency** - Threads and channels

## See Also

- [Type System](type-system.md) - Type inference and checking
- [Grammar](grammar.ebnf) - Language syntax
- Implementation: core/prelude.lib.sigil

---

**Next**: Implement standard library in stdlib/ directory.
