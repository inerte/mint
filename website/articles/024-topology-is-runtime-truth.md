---
title: Topology Is Runtime Truth
date: 2026-03-10
author: Sigil Language Team
slug: topology-is-runtime-truth
---

# Topology Is Runtime Truth

Sigil topology is the canonical declaration of a project's external runtime
dependencies.

Not config.
Not deployment.
Not orchestration.

Topology answers:
- what external things exist
- what their logical names are
- what environment names the project recognizes

Config answers:
- how one environment binds those logical dependencies

## The Practical Shape

A topology-aware project now looks like this:

```text
src/topology.lib.sigil
config/test.lib.sigil
config/production.lib.sigil
```

`src/topology.lib.sigil`:

```sigil
i stdlib⋅topology

c mailerApi=(stdlib⋅topology.httpService("mailerApi"):stdlib⋅topology.HttpServiceDependency)
c production=(stdlib⋅topology.environment("production"):stdlib⋅topology.Environment)
c test=(stdlib⋅topology.environment("test"):stdlib⋅topology.Environment)
```

`config/test.lib.sigil`:

```sigil
i src⋅topology
i stdlib⋅config

c bindings=(stdlib⋅config.bindings([
  stdlib⋅config.bindHttp("http://127.0.0.1:45110",src⋅topology.mailerApi)
],[]):stdlib⋅config.Bindings)
```

Application code still does not know the base URL:

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

## Why This Is Better

This split removes one of the worst forms of runtime ambiguity.

Before, one file mixed:
- dependency identity
- environment names
- concrete URLs and ports
- sometimes env-var names too

Now Sigil makes the separation explicit:
- topology says what exists
- config says how one environment binds it

That is easier for:
- humans
- validators
- code generators
- LLM agents

## The Constraint That Matters

Sigil now treats `process.env` as a config boundary.

That means:
- `process.env` is allowed in `config/*.lib.sigil`
- `process.env` is rejected in ordinary application modules

This matters because otherwise topology becomes optional theater.
If any module can read `MAILER_API_URL` directly, then the declared topology is
just a suggestion.

## Runtime Selection Is Explicit

Sigil also does not guess the environment:

```bash
sigil validate projects/topology-http --env test
sigil run projects/topology-http/src/getClient.sigil --env test
```

No implicit `local`.
No implicit `test`.

If the project is topology-aware, `--env` is required.

## The PL Version

The more formal statement is:

- topology declares logical dependency identities
- config materializes one environment's bindings for those identities
- ambient env access is restricted to config modules
- application code is forced to route external access through dependency handles

That is a stricter and much more machine-readable runtime model.
