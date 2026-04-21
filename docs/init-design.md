# floresta — init design

## What floresta receives at boot

`tronco` spawns floresta as PID 1 with the following initial state:

- Its `VSpaceCap` — its own address space.
- A `CpuCap` with `SchedClass::Interactive` and full budget.
- A `CapFactoryCap` — the meta-cap that lets floresta mint other caps from pooled kernel resources (memory, CPU slices, IRQ lines).
- A pointer to the packed `BootManifest` as a `StorePathCap` mapped read-only.
- A `Cap<EndpointCap<SystemConsole>>` for kernel-log output (replaces early printk).
- Its attestation chain: `[H₀(tronco), H₁(floresta)]`.

floresta does not receive any driver caps or service caps. It manufactures them from the `CapFactoryCap` and hands them down to drivers/services at spawn.

## Boot sequence

```
1. Parse BootManifest (zero-copy — manifest is a [[repr(C)]] struct in mapped memory).
2. Verify manifest self-hash matches raizame::BlakeHash in the cap bag.
3. For each DriverSpec in topologically-sorted order:
      a. Mint caps declared in :caps-requested from CapFactoryCap.
      b. Compute spawn cap-bag digest.
      c. proc_spawn(image=driver.image_hash, cap_bag=caps, vspace=new).
      d. Record the returned ServiceCap<Driver> in the running table.
      e. Wait for driver to publish its EndpointCap<DriverProtocol>.
4. Repeat step 3 for platform services (jabuti-store, kensa-agent, etc.).
5. Repeat step 3 for user services.
6. Enter reconcile loop (see convergence-loop.md).
```

## Spawn mechanics

A spawn is four typed syscalls:

```rust
// 1. Allocate a VSpace for the child.
let vspace = casca.mem_alloc(pages=N, Rights::ReadWrite)?;
// 2. Mint the caps the child needs.
let caps = mint_from_request(&spec.caps_requested)?;
// 3. Open the image.
let image_mem = casca.store_open(&spec.image_path)?;
// 4. Spawn.
let service_cap = casca.proc_spawn(image_mem, caps, vspace)?;
```

Attestation chain extension is done kernel-side in `proc_spawn`. floresta does not compute `Hₙ` itself.

## Dependency resolution

`DriverSpec` and `ServiceSpec` both carry `depends_on: &[&str]`. floresta topologically sorts before spawning. Cycles fail at manifest-parse time in `forja`, not at runtime — the manifest is invariant-checked at compile.

## Cap revocation on exit

When a service exits (clean or crashed):

```rust
1. tronco notifies floresta via its EndpointCap<Supervisor>.
2. floresta receives ExitNotification { service: ServiceCap, reason: ExitReason }.
3. floresta revokes all caps it granted to that service (cap_revoke × N).
4. Per :restart policy:
     - Never → record terminal failure, update observed state.
     - OnFailure + Clean exit → record clean exit.
     - OnFailure + Panic/Denied → enqueue restart.
     - Always → enqueue restart regardless.
5. On next reconcile tick, apply the enqueued actions.
```

Cap revocation is *always* done before restart. There is no path where a
restarted service inherits caps from the previous incarnation.

## Error taxonomy

Spawn can fail. floresta handles each failure explicitly:

| Failure | Response |
|---------|----------|
| `Denied::NoCap` | `CapFactoryCap` exhausted for this cap kind — log, retry on next tick |
| `Denied::OutOfMemory` | System-wide pressure — trigger shrink of lowest-priority services |
| `Denied::ChainOverflow` | Manifest bug (spawn-tree too deep) — mark service permanently-failed |
| `Denied::InvalidArgument` | Manifest mismatch — mark permanently-failed, surface via MCP endpoint |

No failure causes floresta itself to exit. floresta exit = kernel panic.

## What floresta does *not* do at init

- No filesystem mount. `jabuti-store` serves `/nix/store/` and is a service, not a kernel thing.
- No user session setup. Users are a service-graph concept, handled Phase 5 by `kenshou-on-brasa`.
- No network config. `galho-virtio-net` configures itself from declared caps.
- No logging framework setup. Services emit to `SystemConsole` via the endpoint cap they received.
