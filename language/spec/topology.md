# Topology Specification

## Purpose

Sigil topology defines the canonical representation of external runtime
dependencies for topology-aware projects.

Topology is declaration only.
Concrete environment worlds live in config modules.

## Canonical Files

A topology-aware project uses:

```text
src/topology.lib.sigil
config/<env>.lib.sigil
```

`src/topology.lib.sigil` is the canonical source of truth for:
- declared dependency handles
- declared environment names

`config/<env>.lib.sigil` is the canonical source of truth for:
- one selected environment's runtime world

## Topology Surface

`§topology` defines:

```sigil decl §topology
t Environment=Environment(String)
t HttpServiceDependency=HttpServiceDependency(String)
t TcpServiceDependency=TcpServiceDependency(String)

λenvironment(name:String)=>Environment
λhttpService(name:String)=>HttpServiceDependency
λtcpService(name:String)=>TcpServiceDependency
```

`†runtime` and world entry roots define the canonical env surface:

```sigil decl †runtime
t World={clock:†clock.ClockEntry,fs:†fs.FsEntry,http:[†http.HttpEntry],log:†log.LogEntry,process:†process.ProcessEntry,random:†random.RandomEntry,tcp:[†tcp.TcpEntry],timer:†timer.TimerEntry}

λworld(clock:†clock.ClockEntry,fs:†fs.FsEntry,http:[†http.HttpEntry],log:†log.LogEntry,process:†process.ProcessEntry,random:†random.RandomEntry,tcp:[†tcp.TcpEntry],timer:†timer.TimerEntry)=>World
```

## Compile-Time Rules

### Topology declaration location

Calls to these constructors are only valid in `src/topology.lib.sigil`:
- `§topology.httpService`
- `§topology.tcpService`
- `§topology.environment`

### World entry location

Calls to `†http.*` and `†tcp.*` entry constructors are only valid in:

- `config/*.lib.sigil`
- test-local `world { ... }` clauses

### Ambient env access

`process.env` access is only valid in `config/*.lib.sigil`.

It is invalid in:
- `src/topology.lib.sigil`
- ordinary application modules
- tests
- any other project source file

### Dependency-aware API usage

Topology-aware HTTP/TCP APIs require dependency handles:
- `§httpClient.*` requires `HttpServiceDependency`
- `§tcpClient.*` requires `TcpServiceDependency`

The compiler rejects:
- raw URLs passed to topology-aware HTTP client APIs
- raw host/port values passed to topology-aware TCP client APIs
- dependency kind mismatches

## Validate-Time Rules

Validation is environment-specific.

For selected environment `<env>`:
- `src/topology.lib.sigil` must exist
- `<env>` must be declared in topology
- `config/<env>.lib.sigil` must exist
- `config/<env>.lib.sigil` must export `world`
- `world` must provide all primitive effect entries
- every declared dependency must appear exactly once in `world`
- no undeclared dependencies may appear in `world`
- dependency names must be unique in topology

## Execution Model

Topology-aware commands require an explicit environment:

```bash
sigil validate <project> --env <name>
sigil run <file> --env <name>
sigil test <path> --env <name>
```

Sigil does not provide an implicit default environment for topology-aware
projects.

## Diagnostics

Topology diagnostics use `SIGIL-TOPO-*`.

Current codes include:
- `SIGIL-TOPO-MISSING-MODULE`
- `SIGIL-TOPO-MISSING-CONFIG-MODULE`
- `SIGIL-TOPO-INVALID-CONFIG-MODULE`
- `SIGIL-TOPO-ENV-REQUIRED`
- `SIGIL-TOPO-ENV-NOT-FOUND`
- `SIGIL-TOPO-ENV-ACCESS-LOCATION`
- `SIGIL-TOPO-CONSTRUCTOR-LOCATION`
- `SIGIL-TOPO-RAW-ENDPOINT-FORBIDDEN`
- `SIGIL-TOPO-DEPENDENCY-KIND-MISMATCH`
- `SIGIL-TOPO-INVALID-HANDLE`
- `SIGIL-TOPO-DUPLICATE-DEPENDENCY`
- `SIGIL-TOPO-DUPLICATE-BINDING`
- `SIGIL-TOPO-MISSING-BINDING`
- `SIGIL-TOPO-BINDING-KIND-MISMATCH`
