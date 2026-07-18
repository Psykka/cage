# cage

> Declarative sandbox for isolating AI agents and dev environments, built on
> top of `bwrap`. Focus: **deny-by-default** networking + filesystem, with
> allowlisted egress and transparent, controlled access to host services.

Written in **Rust** — the core (exec, fds, unix sockets, child-process
lifecycle) maps cleanly onto the standard library and a couple of mature
crates. CLI only; no TUI planned.

> **Status:** active early-stage development. See [ROADMAP.md](./ROADMAP.md)
> for the detailed per-phase progress.

---

## Why

Running AI agents (Claude, Codex, etc.) or dev environments with unrestricted
access to your network and filesystem is a risk. `cage` isolates execution in
a namespace with:

- **Network at zero by default** — no host is reachable unless explicitly
  allowed.
- **Restricted filesystem** — only the declared binds (ro/rw) exist inside
  the cage.
- **Controlled bridge** to the outside world (allowlisted proxy) and to host
  services (transparent forward), never direct access.
- **Config that's untrusted until proven** — a malicious repo can't design
  its own cage.

## Non-negotiable principles

1. **The cage reaches zero by default.** `--unshare-all`, no `--share-net`.
   Every bit of access is a unix-socket bridge punched through the wall on
   purpose.
2. **The cage's `localhost` ≠ the host's `localhost`.** The isolated netns
   has its own empty loopback. A host service is reachable only if
   deliberately exposed.
3. **External and internal are the same mechanism** (a unix-socket bridge),
   differing only in what sits on the far side: a filtering proxy (external)
   vs. a transparent forward (internal).
4. **`.cagerc.toml` is inert until trusted.** An untrusted repo writes the
   `.cagerc.toml`, so it must not be able to design its own cage. `cage
   allow` (hash-pinned, direnv-style) is what separates "security tool" from
   "pretty wrapper".
5. **Global ceiling.** `~/.config/cage/config` defines the maximum. The
   repo's `.cagerc.toml` can only *tighten* — never loosen.
6. **Don't reimplement namespaces.** `cage` orchestrates, `bwrap` executes.
   (Moving to direct `clone()`/`unshare()` is a long-term goal, not MVP.)

## Install

Requires [`bubblewrap`](https://github.com/containers/bubblewrap) (`bwrap`)
on `PATH` and unprivileged userns enabled in the kernel.

```sh
git clone <repo>
cd cage
cargo build --release
```

## Usage

`cage init` Scaffold a commented `.cagerc.toml` in the current repo     
`cage allow` Mark the current `.cagerc.toml` as trusted (hash-pinned)      
`cage [-p/--profile <name>]` Enter the cage — interactive shell, venv-style, until `exit`    
`cage run [-p/--profile <name>] -- <command>` Run a single command in the cage and leave (CI/script flow)      
`cage check` Preflight check: does bwrap exist? userns enabled? binds present?      
`cage log` Show allow/deny from the last run's proxy     

The bare `cage` (shell mode) and `cage run` share the same argv compiler —
the only difference is whether the final process is an interactive shell or
the user's command.

## Configuration (`.cagerc.toml`)

```toml
[cage]
network = "deny"                       # deny-by-default, always

[filesystem]
ro = ["/usr", "/bin", "/lib"]
rw = ["."]                             # only the project directory

[env]
unset = ["SSH_AUTH_SOCK"]

[external]                             # internet → filtering CONNECT proxy
allow = ["api.anthropic.com:443"]

[internal]                             # host services → transparent forward
expose = ["127.0.0.1:5432", "127.0.0.1:11434"]

[agents.claude]                        # profiles inherit from [base] and only ADD
external.allow = ["api.anthropic.com:443"]
env.pass = ["ANTHROPIC_API_KEY"]

[agents.codex]
external.allow = ["api.openai.com:443"]
env.pass = ["OPENAI_API_KEY"]
```

`~/.config/cage/config` defines the global ceiling; the repo config is
always a subset of it, never an escalation.

## Roadmap

Development follows incremental phases:

| Phase | Goal |
|-------|------|
| 0 — Foundation | The repo builds and runs a "hello from cage" |
| 1 — The argv compiler | Declarative config → correct bwrap line, no networking |
| 2 — External bridge | The cage talks only to allowlisted hosts |
| 3 — Internal bridge | Expose Postgres/Ollama/etc. from the host transparently |
| 4 — Shell mode | Enter and live in the cage, like a real, actually isolated `venv` |
| 5 — Trust model | A malicious `.cagerc.toml` can't design its own cage |

Full details, per-phase checklists, and milestones are in
[ROADMAP.md](./ROADMAP.md).

## Runtime dependencies

- `bwrap` (for now). `cage check` checks for it.
- Unprivileged userns must be enabled in the kernel.

## License

Licensed under the [Apache License, Version 2.0](./LICENSE).
