//! # floresta — brasa userspace init + convergence orchestrator
//!
//! PID 1. The first userspace process spawned by `tronco`. Owns the service
//! graph and the continuous reconciliation loop.
//!
//! See [`docs/init-design.md`](../docs/init-design.md) and
//! [`docs/convergence-loop.md`](../docs/convergence-loop.md).

#![cfg_attr(not(feature = "std"), no_std)]

use heapless::Vec;
use raiz::{Cap, Denied};

/// Boot manifest produced by `forja` from a `(defsystem …)` form. Packed
/// into the kernel image by `semente`; handed to floresta via the cap bag.
#[derive(Debug)]
pub struct BootManifest<'a> {
    pub system_name: &'a str,
    pub arch: Arch,
    pub drivers: &'a [DriverSpec<'a>],
    pub services: &'a [ServiceSpec<'a>],
    pub attest: AttestPolicy<'a>,
    pub converge: ConvergeMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arch {
    Aarch64,
    X86_64,
}

#[derive(Debug)]
pub struct DriverSpec<'a> {
    pub name: &'a str,
    pub image_hash: raizame::BlakeHash,
    pub depends_on: &'a [&'a str],
    pub caps_requested: &'a [CapRequest<'a>],
}

#[derive(Debug)]
pub struct ServiceSpec<'a> {
    pub name: &'a str,
    pub image_hash: raizame::BlakeHash,
    pub depends_on: &'a [&'a str],
    pub caps_granted: &'a [CapRequest<'a>],
    pub restart: RestartPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

/// A declaration that a driver or service needs a cap of the given kind
/// with the given refinement at spawn time. The cap is granted by floresta
/// from its own held caps at spawn-and-check time.
#[derive(Debug)]
pub enum CapRequest<'a> {
    Mmio { device_bound: bool },
    Dma { size_bytes: u64 },
    Irq { line: Option<u16> },
    Net { binds: &'a [NetBind] },
    StoreRead { path_hash: raizame::BlakeHash },
    CpuBudget { cores: u8, class: SchedClass },
}

#[derive(Clone, Copy, Debug)]
pub enum NetBind {
    Tcp(u16),
    Udp(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedClass {
    Interactive,
    Batch,
    RealTime,
}

#[derive(Debug)]
pub struct AttestPolicy<'a> {
    pub baseline: &'a str,          // e.g. "fedramp-moderate"
    pub signer: raizame::BlakeHash, // tameshi public-key hash
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConvergeMode {
    /// Boot once, no reconciliation. For static / embedded deployments.
    OneShot,
    /// Reconcile on a periodic tick. The default.
    Continuous { tick_ms: u32 },
}

/// Reconciliation strategy trait. Implementations are provided by plugins
/// registered in the manifest — typed extension point for specialized
/// convergence behaviors (canary rollout, graceful restart, etc.).
pub trait ReconcileStrategy {
    fn converge(&mut self, declared: &BootManifest, observed: &Snapshot) -> Result<Action, Denied>;
}

/// Observed state of the running system at reconcile time.
#[derive(Debug)]
pub struct Snapshot {
    /// Names of services currently running.
    pub running: Vec<&'static str, 64>,
    /// Names of services that exited since the last reconcile.
    pub exited: Vec<(&'static str, ExitReason), 32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitReason {
    Clean,
    Panic,
    ChainOverflow,
    CapRevoked,
    OutOfMemory,
    Denied,
}

/// Actions floresta takes as output of a reconcile cycle.
#[derive(Debug)]
pub enum Action {
    Idle,
    Spawn(&'static str),
    Restart(&'static str),
    Revoke(&'static str),
    Remeasure(&'static str),
}

#[cfg(feature = "testing")]
pub mod testing {
    //! Mock kernel impl of `casca::Casca` for host-side tests.
    //! Phase 1 deliverable.
}
