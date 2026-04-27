# floresta — operator instructions

> **★★★ CSE / Knowable Construction.** This repo operates under **Constructive Substrate Engineering** — canonical specification at [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md). The Compounding Directive (operational rules: solve once, load-bearing fixes only, idiom-first, models stay current, direction beats velocity) is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before non-trivial changes.


brasa's userspace init + convergence orchestrator. PID 1 on every brasa system.

Start with [docs/init-design.md](./docs/init-design.md). The design reference is [brasa ADR-0004](https://github.com/pleme-io/brasa/blob/main/docs/adrs/0004-tatara-lisp-authoring.md).

## Non-negotiables

- **No string paths.** All config is caps. If you find yourself parsing a path string, stop.
- **No signals.** brasa has no signals. Supervision is via typed IPC and `proc_wait`.
- **Attestation chain totality.** Every child floresta spawns carries an extended chain. Do not build code paths that would circumvent this.
- **Cap revocation.** When a service exits or is restarted, floresta revokes its caps before granting them to the new instance. No leaks.

## Architecture

Single Rust crate at Phase 0. May split into `floresta-core` (types) + `floresta-bin` (runtime) if the reconciliation engine grows complex. Author new features in the main crate first; extract only when the split earns itself.

## Relationship to sibling repos

- **brasa** — provides `casca`, `seiva`, `raiz`, `folha`, `raizame`. floresta is a client of all five.
- **galho-virtio-\*** — first services floresta spawns. Each is its own repo.
- **jabuti-store** — Nix store server. Planned; reads mount tables from the manifest.
- **tatara-lisp / forja** — produces the `BootManifest` floresta consumes at boot.

## The reconciliation loop

Implements the 8-phase convergence model (see [brasa architecture.md](https://github.com/pleme-io/brasa/blob/main/docs/architecture.md) and [pleme-io/CLAUDE.md](https://github.com/pleme-io/nix/blob/main/CLAUDE.md)):

```
declare → simulate (skipped at runtime) → prove (cap validity) → remediate
  → render (nothing at this layer) → deploy (spawn) → verify (health probe)
  → reconverge (drift → declare)
```

At runtime, floresta skips simulate/render because those happened at build time in `forja`. What remains is the continuous loop.

## Testing discipline

- Host-side: `cargo test` with the `testing` feature. Mocks `casca::Casca` with a fake kernel.
- Integration: requires a brasa kernel image + kasou or QEMU. Phase 2+.
