---
title: Topology Says What, Config Says How
date: 2026-03-14
author: Sigil Language Team
slug: topology-vs-config
---

# Topology Says What, Config Says How

One easy way to rot a codebase is to mix runtime architecture and runtime
configuration in the same place.

Sigil now draws a hard line:

- topology says **what** the program depends on
- config says **how** one environment binds those dependencies

That sounds small. It is not.

## The Programmer Version

If a project depends on:
- a mailer HTTP service
- a TCP event stream

then `src/topology.lib.sigil` should only say that:

```sigil
i stdlib⋅topology

c eventStream=(stdlib⋅topology.tcpService("eventStream"):stdlib⋅topology.TcpServiceDependency)
c local=(stdlib⋅topology.environment("local"):stdlib⋅topology.Environment)
c mailerApi=(stdlib⋅topology.httpService("mailerApi"):stdlib⋅topology.HttpServiceDependency)
c production=(stdlib⋅topology.environment("production"):stdlib⋅topology.Environment)
```

No URLs.
No ports.
No usernames.
No passwords.
No API keys.

Those belong in config:

```sigil
⟦ config/local.lib.sigil ⟧
i src⋅topology
i stdlib⋅config

c bindings=(stdlib⋅config.bindings([
  stdlib⋅config.bindHttp("http://127.0.0.1:45110",src⋅topology.mailerApi)
],[
  stdlib⋅config.bindTcp(src⋅topology.eventStream,"127.0.0.1",45120)
]):stdlib⋅config.Bindings)
```

Production config can use env vars:

```sigil
⟦ config/production.lib.sigil ⟧
e process

i src⋅topology
i stdlib⋅config

c bindings=(stdlib⋅config.bindings([
  stdlib⋅config.bindHttpEnv(src⋅topology.mailerApi,"MAILER_API_URL")
],[
  stdlib⋅config.bindTcpEnv(src⋅topology.eventStream,"EVENT_STREAM_HOST","EVENT_STREAM_PORT")
]):stdlib⋅config.Bindings)
```

That gives you one place for local literals and one place for env-driven
production wiring, without turning app code into config archaeology.

## Why This Helps In Practice

It removes three common sources of drift:

First, business logic stops smuggling endpoints through strings.

Second, local configuration stops leaking into architectural declarations.

Third, production secrets stop becoming ambient authority for the whole codebase.

The rule is mechanical:
- `process.env` is only legal in `config/*.lib.sigil`
- topology-aware `run`, `test`, and `validate` require `--env`
- every environment declared in topology needs `config/<env>.lib.sigil`

So the toolchain enforces the separation instead of asking people to remember it.

## Why This Helps With LLMs

Agents freelance when the repo leaves multiple plausible places for truth.

If URLs might live in:
- app code
- topology
- README prose
- env vars
- shell scripts

then the model starts reconstructing intent from fragments.

Sigil reduces that room:
- topology is always `src/topology.lib.sigil`
- environment bindings are always `config/<env>.lib.sigil`
- app code always uses `src⋅topology` handles

That is not an AI feature. It is a language/toolchain constraint that happens to
make AI behavior less sloppy.

## The PL Version

This change is fundamentally about separating:

- **declaration** of external dependency identity
- **materialization** of environment-specific runtime binding

Those are different semantic layers.

When a language blurs them, runtime structure becomes partly architectural,
partly operational, and partly ambient. That makes reasoning weaker for humans,
tools, and validators alike.

Sigil now treats:

- topology as a declaration layer
- config as a materialization layer
- `process.env` as a restricted boundary input

That is the right shape for a machine-first language. It turns runtime wiring
from scattered convention into compiler-visible structure.
