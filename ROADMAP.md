# cage — roadmap

> Declarative sandbox for isolating AI agents and dev environments, built on top
> of `bwrap`. Focus: **deny-by-default** networking + filesystem, with
> allowlisted egress and transparent, controlled access to host services.

Written in **Rust** — the core (exec, fds, unix sockets, child-process
lifecycle) maps cleanly onto the standard library and a couple of mature crates.
CLI only; no TUI planned.

---

## Non-negotiable principles

These come before any feature. If a commit violates them, the commit is wrong,
not the rule.

1. **The cage reaches zero by default.** `--unshare-all`. No `--share-net`.
   Every bit of access is a unix-socket bridge punched through the wall on purpose.
2. **The cage's `localhost` ≠ the host's `localhost`.** The isolated netns has
   its own empty loopback. A host service is reachable only if deliberately exposed.
3. **External and internal are the same mechanism** (a unix-socket bridge),
   differing only in what sits on the far side: a filtering proxy (external) vs.
   a transparent forward (internal).
4. **`.cagerc.toml` is inert until trusted.** An untrusted repo writes the `.cagerc.toml`,
   so it must not be able to design its own cage. `cage allow` (hash-pinned,
   direnv-style) is what separates "security tool" from "pretty wrapper".
5. **Global ceiling.** `~/.config/cage/config` defines the maximum. The repo's
   `.cagerc.toml` can only *tighten* — never loosen. Repo config is a subset of the ceiling.
6. **Don't reimplement namespaces.** `cage` orchestrates, `bwrap` executes.
   (Moving to direct `clone()`/`unshare()` is a long-term goal, not MVP.)

---

## Command surface (final target)

| Command                                 | Group | What it does                                                        |
|-----------------------------------------|-------|---------------------------------------------------------------------|
| `cage init`                             | setup | Scaffold a `.cagerc.toml` in the repo, commented section by section |
| `cage [-p/--profile <name>]`            | exec  | **Enter** the cage — interactive shell, venv-style, until `exit`    |
| `cage allow`                            | setup | Mark the current `.cagerc.toml` as trusted (hash-pinned)            |
| `cage run [-p/--profile <name>] -- cmd` | exec  | Run **one** command in the cage and leave (CI/script flow)          |
| `cage check`                            | diag  | Preflight: does bwrap exist? userns enabled? binds present?         |
| `cage log`                              | diag  | Allow/deny from the last run's proxy (flags suspicious attempts)    |

The bare `cage` and `run` share **the same argv compiler**. The only difference
is whether the final process is an interactive shell or the user's command.

---

## `.cagerc.toml` format (target)

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

---

## Phases

### Phase 0 — Foundation
*Goal: the repo builds and runs a "hello from cage".*

- [x] TOML parsing via `toml` + `serde` derive (typed structs, near-free)
- [x] Typed `.cagerc.toml` structs + validation
- [ ] `cage check`: check `bwrap` on PATH, userns enabled, kernel version
- [x] Subcommand routing scaffold (hand-rolled, no `clap`) — dispatch exists,
      no subcommands wired yet
- [x] Wire first command (`cage init` → `Config::init_in`) — "hello from cage"

### Phase 1 — The argv compiler (the heart)
*Goal: declarative config → correct bwrap line. No networking yet.*

- [ ] Merge `[base]` + `[agents.<profile>]` (profile only adds)
- [ ] Apply the global ceiling (`~/.config/cage/config` bounds the repo)
- [ ] Argv generation: ro/rw binds, `--proc`, `--dev`, `--tmpfs`, `--unshare-all`,
      `--die-with-parent`, `--new-session`, `HOME=/work`, env unsets
- [ ] `--dry-run` that prints the compiled argv (essential for debugging everything)
- [ ] `cage run -- cmd` **without networking**: already confines filesystem + process

> Milestone: `cage run -- ls` runs in a cage where `~/.ssh` does not exist. 90%
> of the security value is already here.

### Phase 2 — External bridge (allowlisted egress)
*Goal: the cage talks ONLY to hosts I permit.*

- [ ] CONNECT proxy on the host listening on a unix socket, with an allowlist
      (TLS end-to-end; the proxy only sees the CONNECT hostname)
- [ ] Proxy spawned **per run**, as a child of cage, dying with it
- [ ] Bind the socket inside + socat bridge (`TCP-LISTEN → UNIX-CONNECT`)
- [ ] `HTTPS_PROXY`/`HTTP_PROXY` pointing at the bridge
- [ ] Structural enforcement: no route in the netns → `unset HTTPS_PROXY` doesn't
      save the agent, it simply can't reach anything off the bridge
- [ ] `cage log` reads the proxy's ALLOW/DENY

> Milestone: agent hits `api.anthropic.com` ✅, tries `evil.com` → DENY on the spot.

### Phase 3 — Internal bridge (host's localhost)
*Goal: expose Postgres/Ollama/etc. from the host transparently and under control.*

- [ ] Transparent forward per exposed port (socat on both sides of the socket)
- [ ] Only what's in `[internal].expose` crosses; the rest of the host stays invisible
- [ ] Confirm transparency: client connects to `127.0.0.1:5432` unaware of the bridge
- [ ] Internal-bridge lifecycle tied to the run's

> Milestone: inside the cage, `psql -h 127.0.0.1` connects to the host's Postgres —
> but only because `5432` is on the list. Drop it from the list, it vanishes.

### Phase 4 — `cage` shell mode (the venv mode)
*Goal: enter and live in the cage, like `venv`, but actually isolated.*

- [ ] Interactive child shell inside the cage (doesn't edit the current shell — new world)
- [ ] `$SHELL` detection (bash/zsh/fish), per-shell rcfile
- [ ] Injected prompt: `(cage:<profile>)` in `PS1`, impossible to confuse
- [ ] Bridges (external + internal) come **up before** the prompt appears
      (`exec $SHELL -i` at the end, socat backgrounded first)
- [ ] `exit`/Ctrl-D tears down the namespace cleanly (no `deactivate`, leaving is leaving)

### Phase 5 — Trust model
*Goal: a malicious `.cagerc.toml` can't design its own cage.*

- [ ] `cage allow`: record path + hash of the `.cagerc.toml` as trusted
- [ ] Untrusted `.cagerc.toml` → refuse to run, ask for `cage allow`
- [ ] Hash changed → revoke trust, ask again
- [ ] Global ceiling actually enforced (repo never escalates past it)
- [ ] Warnings on sensitive binds (`~/.ssh`, `~/.aws`, `~/.config`, the whole `$HOME`)

> Without this phase the tool gives a false sense of security and **worsens** your
> posture. Do not skip.

---

## Field notes

- **Runtime deps:** `bwrap` (for now). `check` should check both.
  Unprivileged userns must be enabled in the kernel.