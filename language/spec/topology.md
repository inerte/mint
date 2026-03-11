# Topology Specification

## Purpose

Sigil topology defines the canonical representation of external runtime
dependencies for a topology-aware project.

Topology is part of compile-time and validate-time behavior. It is not an
advisory metadata format.

## Canonical Topology Module

A topology-aware project defines:

```text
src/topology.lib.sigil
```

This module is the canonical source of truth for:
- declared dependency handles
- declared environments
- environment bindings

Topology constructors are restricted to this module.

## Dependency Kinds

Topology v1 defines two concrete dependency kinds:

- `HttpServiceDependency`
- `TcpServiceDependency`

Dependency identities are logical names. They are not concrete endpoints.

## Topology Surface

`stdlib⋅topology` defines the canonical surface:

```sigil
t BindingValue=EnvVar(String)|Literal(String)
t Environment={httpBindings:[HttpBinding],name:String,tcpBindings:[TcpBinding]}
t HttpBinding={baseUrl:BindingValue,dependency:HttpServiceDependency}
t HttpServiceDependency=HttpServiceDependency(String)
t PortBindingValue=EnvVarPort(String)|LiteralPort(Int)
t TcpBinding={dependency:TcpServiceDependency,host:BindingValue,port:PortBindingValue}
t TcpServiceDependency=TcpServiceDependency(String)

λbindHttp(baseUrl:String,dependency:HttpServiceDependency)→HttpBinding
λbindHttpEnv(dependency:HttpServiceDependency,envVar:String)→HttpBinding
λbindTcp(dependency:TcpServiceDependency,host:String,port:Int)→TcpBinding
λbindTcpEnv(dependency:TcpServiceDependency,hostEnvVar:String,portEnvVar:String)→TcpBinding
λenvironment(httpBindings:[HttpBinding],name:String,tcpBindings:[TcpBinding])→Environment
λhttpService(name:String)→HttpServiceDependency
λtcpService(name:String)→TcpServiceDependency
```

## Compile-Time Rules

### Dependency-aware API usage

Topology-aware HTTP/TCP APIs require dependency handles:

- `stdlib⋅httpClient.get` and related helpers require `HttpServiceDependency`
- `stdlib⋅tcpClient.send` and related helpers require `TcpServiceDependency`

The compiler rejects:
- raw URLs passed to topology-aware HTTP client APIs
- raw host/port values passed to topology-aware TCP client APIs
- handle kind mismatches

### Constructor location

Calls to topology constructors are only valid in `src/topology.lib.sigil`:
- `httpService`
- `tcpService`
- `environment`
- `bindHttp`
- `bindHttpEnv`
- `bindTcp`
- `bindTcpEnv`

### Diagnostics

Topology compile-time diagnostics use `SIGIL-TOPO-*`.

Current topology codes:
- `SIGIL-TOPO-MISSING-MODULE`
- `SIGIL-TOPO-RAW-ENDPOINT-FORBIDDEN`
- `SIGIL-TOPO-DEPENDENCY-KIND-MISMATCH`
- `SIGIL-TOPO-INVALID-HANDLE`
- `SIGIL-TOPO-CONSTRUCTOR-LOCATION`
- `SIGIL-TOPO-DUPLICATE-DEPENDENCY`
- `SIGIL-TOPO-DUPLICATE-BINDING`
- `SIGIL-TOPO-MISSING-BINDING`
- `SIGIL-TOPO-BINDING-KIND-MISMATCH`
- `SIGIL-TOPO-ENV-NOT-FOUND`

## Validate-Time Rules

Topology validation is environment-specific.

For a selected environment:
- the environment must exist
- every declared dependency used by the project must be bound
- binding kind must match dependency kind
- dependency names must be unique across the topology module
- bindings must be unique within one environment

Environment names are user-defined. Sigil does not reserve or standardize names
such as `local`, `test`, or `prod`.

## Environment Bindings

Bindings may be literal or env-based.

HTTP:
- literal base URL: `bindHttp`
- env-derived base URL: `bindHttpEnv`

TCP:
- literal host/port: `bindTcp`
- env-derived host/port: `bindTcpEnv`

Environment variable references belong only in the binding layer, not in
application code.

## HTTP and TCP Client Integration

Topology-aware client usage is canonical:

```sigil
stdlib⋅httpClient.get(src⋅topology.mailerApi,headers,"/health")
stdlib⋅tcpClient.send(src⋅topology.eventStream,"ping")
```

Application code must not carry concrete endpoint strings for topology-backed
dependencies.

## Execution Model

`sigil validate --env <name>` validates topology for one selected environment.

`sigil run --env <name>` and `sigil test --env <name>` validate topology before
runtime execution.

Ordinary compile still performs usage-shape checks even without environment
selection.
