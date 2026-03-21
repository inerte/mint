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

## Automatic Imports

The prelude is automatically imported into every Sigil module. No explicit import needed.

## Core Types

### ConcurrentOutcome[T,E]

Implicit core prelude sum type:

```sigil decl core::prelude
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

### Implemented `stdlib::list` Functions

```sigil decl stdlib::list
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

- use `stdlib::list.all` for universal checks
- use `stdlib::list.any` for existential checks
- use `stdlib::list.countIf` for predicate counting
- use `map` for projection
- use `filter` for filtering
- use `stdlib::list.find` for first-match search
- use `stdlib::list.flatMap` for flattening projection
- use `reduce ... from ...` or `stdlib::list.fold` for reduction
- use `stdlib::list.reverse` for reversal

The validator rejects exact recursive clones of `all`, `any`, `map`, `filter`,
`find`, `flatMap`, `fold`, and `reverse`, rejects `#(xs filter pred)` in favor of
`stdlib::list.countIf`, and rejects recursive result-building of the form
`self(rest)⧺rhs`. These are narrow AST-shape rules, not a general complexity
prover.

### Implemented `stdlib::numeric` Helpers

```sigil decl stdlib::numeric
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

## String Operations

```sigil decl stdlib::string
λcharAt(idx:Int,s:String)=>String
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
λtrim(s:String)=>String
λunlines(lines:[String])=>String
```

## File and Process Operations

### Implemented `stdlib::file` Functions

```sigil decl stdlib::file
λappendText(content:String,path:String)=>!IO Unit
λexists(path:String)=>!IO Bool
λlistDir(path:String)=>!IO [String]
λmakeDir(path:String)=>!IO Unit
λmakeDirs(path:String)=>!IO Unit
λmakeTempDir(prefix:String)=>!IO String
λreadText(path:String)=>!IO String
λremove(path:String)=>!IO Unit
λremoveTree(path:String)=>!IO Unit
λwriteText(content:String,path:String)=>!IO Unit
```

`makeTempDir(prefix)` creates a fresh temp directory and returns its absolute
path. Cleanup remains explicit through `removeTree`.

### Implemented `stdlib::process` Types and Functions

```sigil decl stdlib::process
t Command={argv:[String],cwd:Option[String],env:{String↦String}}
t RunningProcess={pid:Int}
t ProcessResult={code:Int,stderr:String,stdout:String}

λcommand(argv:[String])=>Command
λexit(code:Int)=>!IO Unit
λwithCwd(command:Command,cwd:String)=>Command
λwithEnv(command:Command,env:{String↦String})=>Command
λrun(command:Command)=>!IO ProcessResult
λstart(command:Command)=>!IO RunningProcess
λwait(process:RunningProcess)=>!IO ProcessResult
λkill(process:RunningProcess)=>!IO Unit
```

Process rules:
- command execution is argv-based only
- `withEnv` overlays explicit variables on top of the inherited environment
- non-zero exit codes are reported in `ProcessResult.code`
- `run` captures stdout and stderr in memory
- `kill` is a normal termination request, not a timeout/escalation protocol

### Implemented `stdlib::regex` Types and Functions

```sigil decl stdlib::regex
t Regex={flags:String,pattern:String}
t RegexError={message:String}
t RegexMatch={captures:[String],end:Int,full:String,start:Int}

λcompile(flags:String,pattern:String)=>Result[Regex,RegexError]
λfind(input:String,regex:Regex)=>Option[RegexMatch]
λisMatch(input:String,regex:Regex)=>Bool
```

Regex rules:
- v1 semantics follow JavaScript `RegExp`
- `compile` validates both flags and pattern before returning `Ok`
- `find` returns the first match only
- unmatched capture groups are returned as empty strings in `captures`

### Implemented `stdlib::time` Additions

```sigil decl stdlib::time
λsleepMs(ms:Int)=>!IO Unit
```

`sleepMs` is the canonical delay primitive for retry loops and harness
orchestration.

## Map Operations

Maps are a core collection concept, and helper functions live in `core::map`.

```sigil decl core::map
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

```sigil decl stdlib::json
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

`stdlib::decode` is the canonical boundary layer from raw `JsonValue` to trusted
internal Sigil values.

```sigil decl stdlib::decode
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
- `stdlib::json` owns raw parsing and inspection.
- `stdlib::decode` owns conversion into trusted internal types.
- `DecodeError.path` records the nested field/index path of the failure.
- If a field may be absent, keep the record exact and use `Option[T]` for that field.
- Sigil does not use open records or partial records for this boundary story.

## Time Operations

```sigil decl stdlib::time
t Instant={epochMillis:Int}
t TimeError={message:String}

λparseIso(input:String)=>Result[Instant,TimeError]
λformatIso(instant:Instant)=>String
λnow()=>!IO Instant
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

The numeric helper surface is owned by `stdlib::numeric`; there is no separate
math module today.

## I/O Operations

All I/O operations have the `!IO` effect.

```sigil decl stdlib::io
λdebug(msg:String)=>!IO Unit
λeprintln(msg:String)=>!IO Unit
λprint(msg:String)=>!IO Unit
λprintln(msg:String)=>!IO Unit
λwarn(msg:String)=>!IO Unit
```

## Module System

### Import Syntax

```sigil module
i stdlib::file

i stdlib::list

i stdlib::path

i stdlib::process
```

### Export Visibility

File extension determines visibility:

**`.lib.sigil` files** (libraries):
- All top-level declarations are automatically visible to importers
- No `export` keyword needed or allowed

**`.sigil` files** (executables):
- Cannot be imported (except by test files in `tests/` directories)
- Have `main()` function

No selective imports, no aliasing, no export lists.

## Standard Library Modules

### core/prelude

Auto-imported. Contains the foundational vocabulary types:
- `Option[T]`
- `Result[T,E]`
- `Some`
- `None`
- `Ok`
- `Err`

### stdlib::file

UTF-8 filesystem helpers:
- `appendText`
- `exists`
- `listDir`
- `makeDir`
- `makeDirs`
- `readText`
- `remove`
- `removeTree`
- `writeText`

### stdlib::path

Filesystem path helpers:
- `basename`
- `dirname`
- `extname`
- `join`
- `normalize`
- `relative`

### stdlib::io

Console and process I/O only (`print`, `println`, `eprintln`, `warn`, `debug`)

### core::map

Dynamic keyed collection helpers over `{K↦V}` values.

### stdlib::numeric

Integer helpers (`abs`, `divmod`, `gcd`, `lcm`, `max`, `min`, `mod`,
predicates like `isEven`, and ranges).

### stdlib::json

Typed JSON parsing and serialization (`JsonValue`, `parse`, `stringify`)

```sigil decl stdlib::json
λparse(input:String)=>Result[JsonValue,JsonError]
λstringify(value:JsonValue)=>String
```

### stdlib::decode

Canonical JSON-to-domain decoding (`Decoder[T]`, `DecodeError`, `run`, `parse`)

```sigil decl stdlib::decode
λrun[T](decoder:Decoder[T],value:JsonValue)=>Result[T,DecodeError]
λparse[T](decoder:Decoder[T],input:String)=>Result[T,DecodeError]
```

### stdlib::time

Time and instant handling (`Instant`, strict ISO parsing, clock access)

```sigil decl stdlib::time
λparseIso(input:String)=>Result[Instant,TimeError]
λformatIso(instant:Instant)=>String
λnow()=>!IO Instant
```

### stdlib::topology

Canonical declaration layer for external HTTP and TCP runtime dependencies.

```sigil decl stdlib::topology
t Environment=Environment(String)
t HttpServiceDependency=HttpServiceDependency(String)
t TcpServiceDependency=TcpServiceDependency(String)

λenvironment(name:String)=>Environment
λhttpService(name:String)=>HttpServiceDependency
λtcpService(name:String)=>TcpServiceDependency
```

### stdlib::config

Canonical binding layer for topology-backed environment config.

```sigil decl stdlib::config
t BindingValue=EnvVar(String)|Literal(String)
t Bindings={httpBindings:[HttpBinding],tcpBindings:[TcpBinding]}
t HttpBinding={baseUrl:BindingValue,dependencyName:String}
t PortBindingValue=EnvVarPort(String)|LiteralPort(Int)
t TcpBinding={dependencyName:String,host:BindingValue,port:PortBindingValue}

λbindHttp(baseUrl:String,dependency:stdlib::topology.HttpServiceDependency)=>HttpBinding
λbindHttpEnv(dependency:stdlib::topology.HttpServiceDependency,envVar:String)=>HttpBinding
λbindTcp(dependency:stdlib::topology.TcpServiceDependency,host:String,port:Int)=>TcpBinding
λbindTcpEnv(dependency:stdlib::topology.TcpServiceDependency,hostEnvVar:String,portEnvVar:String)=>TcpBinding
λbindings(httpBindings:[HttpBinding],tcpBindings:[TcpBinding])=>Bindings
```

### stdlib::httpClient

Canonical text-based HTTP client.

```sigil decl stdlib::httpClient
t Headers={String↦String}
t HttpError={kind:HttpErrorKind,message:String}
t HttpErrorKind=InvalidJson()|InvalidUrl()|Network()|Timeout()|Topology()
t HttpMethod=Delete()|Get()|Patch()|Post()|Put()
t HttpRequest={body:Option[String],dependency:stdlib::topology.HttpServiceDependency,headers:Headers,method:HttpMethod,path:String}
t HttpResponse={body:String,headers:Headers,status:Int,url:String}

λrequest(request:HttpRequest)=>!IO Result[HttpResponse,HttpError]
λget(dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[HttpResponse,HttpError]
λdelete(dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[HttpResponse,HttpError]
λpost(body:String,dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[HttpResponse,HttpError]
λput(body:String,dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[HttpResponse,HttpError]
λpatch(body:String,dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[HttpResponse,HttpError]

λgetJson(dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[JsonValue,HttpError]
λdeleteJson(dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[JsonValue,HttpError]
λpostJson(body:JsonValue,dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[JsonValue,HttpError]
λputJson(body:JsonValue,dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[JsonValue,HttpError]
λpatchJson(body:JsonValue,dependency:stdlib::topology.HttpServiceDependency,headers:Headers,path:String)=>!IO Result[JsonValue,HttpError]
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

### stdlib::httpServer

Canonical request/response HTTP server.

```sigil decl stdlib::httpServer
t Headers={String↦String}
t Request={body:String,headers:Headers,method:String,path:String}
t Response={body:String,headers:Headers,status:Int}

λresponse(body:String,contentType:String,status:Int)=>Response
λok(body:String)=>Response
λjson(body:String,status:Int)=>Response
λnotFound()=>Response
λnotFoundMsg(message:String)=>Response
λserverError(message:String)=>Response
λlogRequest(request:Request)=>!IO Unit
λserve(handler:λ(Request)=>!IO Response,port:Int)=>!IO Unit
```

`serve` is long-lived: once the server is listening, the process remains active
until it is terminated externally.

### stdlib::tcpClient

Canonical one-request, one-response TCP client.

```sigil decl stdlib::tcpClient
t TcpError={kind:TcpErrorKind,message:String}
t TcpErrorKind=Connection()|InvalidAddress()|Protocol()|Timeout()|Topology()
t TcpRequest={dependency:stdlib::topology.TcpServiceDependency,message:String}
t TcpResponse={message:String}

λrequest(request:TcpRequest)=>!IO Result[TcpResponse,TcpError]
λsend(dependency:stdlib::topology.TcpServiceDependency,message:String)=>!IO Result[TcpResponse,TcpError]
```

Semantics:
- requests are UTF-8 text
- the client writes one newline-delimited message and expects one newline-delimited response
- address validation, socket failure, timeout, topology resolution failure, and framing failure return `Err(TcpError)`

### stdlib::tcpServer

Canonical one-request, one-response TCP server.

```sigil decl stdlib::tcpServer
t Request={host:String,message:String,port:Int}
t Response={message:String}

λresponse(message:String)=>Response
λserve(handler:λ(Request)=>!IO Response,port:Int)=>!IO Unit
```

Semantics:
- the server reads one UTF-8 line per connection
- the handler returns one UTF-8 line response
- the server closes each connection after the response is written
- `serve` is long-lived once listening succeeds

### Testing

Testing is built into the language with `test` declarations and the `sigil
test` runner. There is no current `stdlib::test` module surface.

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
- String concatenation in loops is O(n²) - prefer stdlib::string.join when building from parts

### Effect System

Effects are tracked at type level:
- `!IO` - Input/output operations
- `!Test` - Test operations
- Pure functions have no effect annotation

## Future Extensions

Planned for future stdlib versions:

- **stdlib::crypto** - Cryptographic functions
- **stdlib::random** - Random number generation
- **stdlib::stream** - Streaming I/O
- **stdlib::concurrency** - Threads and channels

## See Also

- [Type System](type-system.md) - Type inference and checking
- [Grammar](grammar.ebnf) - Language syntax
- Implementation: core/prelude.lib.sigil

---

**Next**: Implement standard library in stdlib/ directory.
