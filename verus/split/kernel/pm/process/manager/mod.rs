// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Include specifications.
include!("mod.spec.rs");

// Include proofs.
include!("mod.proof.rs");

// ProcessManagerInner: abstract verification model of the inner process manager.
// Originally in process_manager.rs submodule, included here so that all exec
// functions are visible in mod.rs scope (matching the original source layout).
include!("process_manager.rs");

// ProcessManagerUnsafeState (models ProcessManager): abstract verification model
// of the global unsafe wrapper. Originally in process_manager_unsafe.rs submodule,
// included here for the same reason.
include!("process_manager_unsafe.rs");
