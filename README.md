# floresta (forest)

> The userspace init + convergence orchestrator for [brasa](https://github.com/pleme-io/brasa). Reads a typed `BootManifest` produced from `(defsystem …)`, spawns the declared service graph, runs a continuous FluxCD-style reconciliation loop.

**Brazilian-Portuguese for *forest*** — the ecosystem of processes that grows out of brasa's kernel. Where every `folha` (leaf / process) hangs off its `galho` (branch / driver), all anchored to the one `tronco` (trunk / kernel).

**Status:** Phase 0 — Design. No binary runs yet. Scaffolded alongside brasa to anchor the ADR-0004 references.

**License:** MIT.

## What this is

floresta is brasa's PID 1. It is the first userspace process spawned by `tronco` after boot. It owns the service graph and the convergence loop.

Floresta is not daemon-style init (no runlevels, no `/etc/init.d`). It is declarative: you hand it a typed `BootManifest`, it converges the running system toward the declaration. Drifts auto-reconcile.

## Relationship to brasa

```
semente (bootloader)
  │
  ▼ measures + hands off BootInfo
tronco (kernel)
  │
  ▼ spawns with attestation chain
floresta (this repo, PID 1)
  │
  ▼ reads (defsystem …) manifest, spawns:
  ├── galho-virtio-console (first driver — stdout)
  ├── galho-virtio-net     (second — networking)
  ├── galho-virtio-blk     (third — persistence)
  ├── jabuti-store         (serves /nix/store/ paths)
  └── <user-declared services>
```

floresta holds one `BootManifest` cap and the root `CapFactoryCap`. Every other cap in the running system flows through floresta at spawn time.

## Reading order

1. [docs/init-design.md](./docs/init-design.md) — how floresta reads the manifest and spawns services
2. [docs/convergence-loop.md](./docs/convergence-loop.md) — the continuous reconciliation model

## What floresta does

1. **Boot handoff.** Receives the `BootManifest` pointer and initial cap bag from `tronco`.
2. **First phase — drivers.** Spawns the declared drivers in dependency order. Each driver receives its `MmioCap`, `DmaCap`, `IrqCap<N>` grants.
3. **Second phase — platform services.** Spawns `jabuti-store`, the attestation agent, the convergence agent.
4. **Third phase — user services.** Spawns every `(defservice …)` in dependency order. Each service receives caps declared in `:caps-granted`.
5. **Reconcile loop.** On a 5-second tick (configurable), compares declared state to observed state. Remediates drift: restarts failed services, reaps zombies, re-grants lost caps where semantics allow.
6. **Signal-free supervision.** Services that exit with `Denied::ChainOverflow` or panic are restarted per their `:restart` policy (`:never`, `:on-failure`, `:always`).

## What floresta does not do

- Parse any string path. All config is caps.
- Provide a shell. Operators interact via the floresta MCP endpoint (`fumi`-like client).
- Mount filesystems. The only namespace is `/nix/store/` served by `jabuti-store`.
- Manage users. There are no users at the kernel level.
- Run cron. Scheduled tasks are services with typed `:schedule` fields.

## Build

Phase 0: nothing builds yet.

When it does:

```bash
nix develop
cargo check                     # typecheck against brasa target-triple stubs
cargo build --target aarch64-unknown-brasa    # Phase 1 target (triple tbd)
```

## Dependencies

- [`casca`](https://github.com/pleme-io/brasa/tree/main/crates/casca) — syscall ABI
- [`seiva`](https://github.com/pleme-io/brasa/tree/main/crates/seiva) — typed IPC
- [`folha`](https://github.com/pleme-io/brasa/tree/main/crates/folha) + `folha::rt` — userspace runtime (replaces libc)
- [`raiz`](https://github.com/pleme-io/brasa/tree/main/crates/raiz) — cap types
- `tatara-lisp` — consumes the `BootManifest` produced by `forja`

## License

MIT. See [LICENSE](./LICENSE).
