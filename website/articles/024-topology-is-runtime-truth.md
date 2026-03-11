---
title: Topology Is Runtime Truth
date: 2026-03-10
author: Sigil Language Team
slug: topology-is-runtime-truth
---

# Topology Is Runtime Truth

Sigil now has a canonical way to describe external runtime dependencies.

This is not Terraform.
This is not Kubernetes.
This is not a service registry.

It is a stricter and smaller idea:

- application code names logical dependencies
- topology binds those dependencies per environment
- the compiler and validator enforce that application code does not bypass that model

## The Practical Problem

In most projects, runtime dependencies are scattered across:

- raw URLs in code
- raw host/port pairs in code
- environment variables
- README prose
- Compose files
- deploy configs

That means both humans and tools have to reconstruct the system by archaeology.

Sigil prefers one machine-readable source of truth.

## What Topology Looks Like

A topology-aware project defines:

```text
src/topology.lib.sigil
```

That file exports:
- dependency handles
- environment bindings

Example:

```sigil
i stdlib⋅topology

c mailerApi=(stdlib⋅topology.httpService("mailerApi"):stdlib⋅topology.HttpServiceDependency)

c test=(stdlib⋅topology.environment([
  stdlib⋅topology.bindHttp("http://127.0.0.1:45110",stdlib⋅topology.httpService("mailerApi"))
],"test",[]):stdlib⋅topology.Environment)
```

Application code does not know the base URL:

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

The same idea works for TCP:

```sigil
i src⋅topology
i stdlib⋅tcpClient

λmain()→!IO String match stdlib⋅tcpClient.send(src⋅topology.eventStream,"ping"){
  Ok(response)→response.message|
  Err(error)→error.message
}
```

## Why This Fits Sigil

Sigil is machine-first.

That means:
- one canonical source of truth beats several adjacent sources
- runtime structure should be visible to the toolchain
- application code should not smuggle infrastructure details through raw strings

Topology fits that vision directly.

Instead of:
- “the mailer URL is probably in an env var somewhere”

Sigil can say:
- this project depends on `mailerApi`
- `test` binds it here
- `prod` binds it differently
- application code must use that handle

That is much better for:
- compile-time enforcement
- validation
- testing
- future Claude Code sessions

## Compile-Time vs Validate-Time

Sigil splits topology enforcement into two layers.

### Compile-time

Compile-time checks usage shape:

- HTTP client APIs require `HttpServiceDependency`
- TCP client APIs require `TcpServiceDependency`
- raw URLs and raw host/port values are rejected for topology-aware calls
- wrong dependency kinds are rejected

That is the shift-left part.

### Validate-time

Environment validation checks completeness:

- the selected environment exists
- every required dependency is bound
- binding kinds match dependency kinds
- duplicate dependency names and duplicate bindings are rejected

Use:

```bash
sigil validate --env test projects/topology-http
```

and:

```bash
sigil run --env test projects/topology-http/src/getClient.sigil
```

## Tests Are Just Environments

Sigil does not need a special “mock topology” model first.

Tests are just environments:
- same dependency identity
- different concrete binding

That keeps the mental model simple.

If production code depends on `mailerApi`, test code also depends on `mailerApi`.
The `test` environment tells Sigil where that dependency lives during tests.

## The First Two Projects

This feature is already dogfooded with two small projects.

### `projects/topology-http`

This project depends on a Sigil HTTP service through `mailerApi`.

It proves:
- topology-backed HTTP handles
- environment-based URL binding
- route-relative client calls
- compiler rejection of raw HTTP endpoints in topology-aware code

### `projects/topology-tcp`

This project depends on a Sigil TCP service through `eventStream`.

It proves:
- topology-backed TCP handles
- environment-based host/port binding
- non-HTTP runtime dependencies using the same model

Together they show that topology is not “just for REST.”
It is a general runtime dependency declaration model for Sigil projects.

## The PL Version

If you want the more formal phrasing:

- topology introduces **typed logical dependency identities**
- environments define **concrete bindings** for those identities
- application code is prohibited from bypassing those identities with raw endpoint values
- compile-time enforces dependency-handle usage
- validate-time enforces environment completeness and binding compatibility

That is exactly the kind of machine-readable runtime structure Sigil should own.
