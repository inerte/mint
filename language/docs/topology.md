# Topology

Sigil topology is the canonical, compiler-visible declaration of a project's
external runtime dependencies.

Topology is not:
- Terraform
- Kubernetes
- service discovery
- deployment orchestration

Topology is:
- the logical identities of the external services your project depends on
- the concrete bindings for those dependencies per environment
- a compile/validate contract that application code must obey

## Why Sigil Has Topology

Without topology, runtime dependencies are usually scattered across:
- `README` prose
- environment variables
- Docker Compose files
- deployment configs
- ad hoc strings in application code

That is bad for both humans and tools.

Sigil prefers one canonical source of truth:
- application code uses typed dependency handles
- topology binds those handles per environment
- the compiler and validator can check that the wiring is real

## Canonical Project Shape

Topology-aware projects define:

```text
src/topology.lib.sigil
```

That module exports:
- dependency handles
- environment declarations

Example:

```sigil
i stdlib⋅topology

c eventStream=(stdlib⋅topology.tcpService("eventStream"):stdlib⋅topology.TcpServiceDependency)
c mailerApi=(stdlib⋅topology.httpService("mailerApi"):stdlib⋅topology.HttpServiceDependency)

c local=(stdlib⋅topology.environment([
  stdlib⋅topology.bindHttp("http://127.0.0.1:45110",stdlib⋅topology.httpService("mailerApi"))
],"local",[
  stdlib⋅topology.bindTcp(eventStream,"127.0.0.1",45120)
]):stdlib⋅topology.Environment)

c test=(stdlib⋅topology.environment([
  stdlib⋅topology.bindHttp("http://127.0.0.1:45110",stdlib⋅topology.httpService("mailerApi"))
],"test",[
  stdlib⋅topology.bindTcp(eventStream,"127.0.0.1",45120)
]):stdlib⋅topology.Environment)

c prod=(stdlib⋅topology.environment([
  stdlib⋅topology.bindHttpEnv(stdlib⋅topology.httpService("mailerApi"),"MAILER_API_URL")
],"prod",[
  stdlib⋅topology.bindTcpEnv(eventStream,"EVENT_STREAM_HOST","EVENT_STREAM_PORT")
]):stdlib⋅topology.Environment)
```

Environment names are flexible. Sigil does not standardize `local`, `dev`,
`staging`, or `prod`. It only validates structure and uniqueness.

## Application Code Uses Handles, Not Endpoints

The purpose of topology is to keep raw runtime endpoints out of business logic.

Canonical HTTP usage:

```sigil
i src⋅topology
i stdlib⋅httpClient

λmain()→!IO String match stdlib⋅httpClient.get(
  src⋅topology.mailerApi,
  stdlib⋅httpClient.emptyHeaders(),
  "/health"
){
  Ok(response)→response.body|
  Err(error)→error.message
}
```

Canonical TCP usage:

```sigil
i src⋅topology
i stdlib⋅tcpClient

λmain()→!IO String match stdlib⋅tcpClient.send(src⋅topology.eventStream,"ping"){
  Ok(response)→response.message|
  Err(error)→error.message
}
```

Non-canonical raw endpoint usage is rejected for topology-aware APIs.

Examples of forbidden application-code patterns:

```sigil
stdlib⋅httpClient.get("http://127.0.0.1:45110",headers,"/health")
stdlib⋅tcpClient.send("127.0.0.1","ping",45120)
```

## Compile-Time vs Validate-Time

Sigil splits topology enforcement deliberately.

### Compile-time

Compile-time checks usage shape:
- topology-aware HTTP/TCP APIs require dependency handles
- raw URLs, raw hosts, raw ports, and ad hoc strings are rejected
- HTTP APIs reject TCP handles
- TCP APIs reject HTTP handles
- topology constructors are restricted to `src/topology.lib.sigil`

### Validate-time

Environment validation checks completeness:
- the selected environment exists
- every used dependency is bound
- binding kinds match dependency kinds
- duplicate dependency names are rejected
- duplicate bindings inside one environment are rejected

Use:

```bash
sigil validate --env test projects/topology-http
```

`sigil run --env ...` and `sigil test --env ...` validate topology before
execution.

## Tests Are Environments

Sigil does not need a separate topology mocking model in v1.

Tests are just another environment:
- same dependency identity
- different concrete bindings

That means test code still refers to:
- `src⋅topology.mailerApi`
- `src⋅topology.eventStream`

and the `test` environment decides where those logical dependencies resolve.

## Current V1 Scope

Topology v1 currently covers:
- `HttpServiceDependency`
- `TcpServiceDependency`

This is intentional.

The goal is to establish one canonical runtime dependency model first, then
extend it later to other concrete dependency kinds like databases or queues.
