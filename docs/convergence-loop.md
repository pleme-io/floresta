# floresta — convergence loop

## Model

floresta implements the final three phases of the pleme-io 8-phase convergence model at runtime:

```
declare → simulate → prove → remediate → render → deploy → verify → reconverge
                                                   ^^^^^^   ^^^^^^   ^^^^^^^^^^
                                                   floresta handles these three.
```

`declare`, `simulate`, `prove`, `remediate`, and `render` happen at build time inside `forja` (for the typed manifest) and `nix build` (for the images). By the time floresta sees a `BootManifest`, the above are guaranteed sound.

At runtime, floresta executes:

- **deploy:** spawn all declared drivers + services in dep-sorted order.
- **verify:** health-probe each service, collect observed state.
- **reconverge:** diff declared vs observed; act (restart / revoke / remeasure / mark-failed).

## The tick

```rust
loop {
    sleep(manifest.converge.tick_ms);
    let observed = snapshot_running_services();
    let actions = strategy.converge(&manifest, &observed)?;
    for action in actions {
        dispatch(action)?;
    }
}
```

Default `tick_ms = 5000` (5 seconds). Interactive deployments may lower it; embedded deployments use `ConvergeMode::OneShot` and do not tick.

## What counts as drift

1. A declared service is not in `observed.running` → spawn it.
2. A running service is not declared → revoke and shut down (unlikely on brasa — nothing else can spawn; this is defense-in-depth for a future floresta-of-florestas topology).
3. A service in `observed.exited` with `RestartPolicy::OnFailure` and a failure reason → restart.
4. A declared service's `image_hash` differs from the running instance's chain link → graceful restart with new image.
5. A service's cap bag drifts (e.g., a held `StorePathCap` points to a path that was rebuilt) → remeasure + graceful restart.

## What floresta does not do

- **No mutation of the manifest at runtime.** If the declared state must change, `forja` produces a new manifest, `semente` reboots with it, and floresta converges to the new shape. This is the FluxCD-via-git-commit model: new manifest = new commit = convergence toward new state.
  - Exception in Phase 4+: a mutable `LiveManifestCap` may allow a privileged upstream (e.g., a declarative-ops agent) to hand floresta a manifest update. This is gated behind ADR-TBD.
- **No canary logic.** Phase 1 floresta is all-or-nothing per service. Canary rollouts, blue-green, etc. land as pluggable `ReconcileStrategy` implementations in Phase 3+.

## Observability

Every action floresta takes is a typed event on its `EventSink` endpoint. A `kensa-agent` subscribes to this sink and exports events to `shinryu` (existing pleme-io observability plane). No log files, no `dmesg` — structured events, end-to-end typed.

## Failure modes

- floresta panic → kernel panic (brasa policy: PID 1 panic halts the system; `sekiban` records the chain hash for post-mortem). No "respawn floresta" logic exists; recovery is reboot with the recorded manifest.
- Strategy returns `Denied::ChainOverflow` → the strategy attempted a spawn that would exceed `MAX_DEPTH=16`. Caught, logged, strategy is flagged; floresta falls back to the default strategy.
- Strategy returns `Denied::OutOfMemory` on every tick → backpressure signal to the system admin via MCP; floresta continues at longer tick intervals until resources recover.

## Related ADRs (in brasa repo)

- ADR-0001 — capability ABI
- ADR-0002 — attestation chain
- ADR-0004 — tatara-lisp authoring (the manifest source language)
- ADR-0005 — Nix store as filesystem (where service images live)
