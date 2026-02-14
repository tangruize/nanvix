// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Process Manager Module Specification.
//
// This module (`mod.rs`) serves as a re-export hub for the process manager
// submodules (`process_manager` and `process_manager_unsafe`). It does not
// define any types or functions of its own.
//
// All spec functions, View types, and invariants are defined in the
// respective submodule spec files:
// - `process_manager.spec.rs`: ProcessManagerInner specs and View type.
// - `process_manager_unsafe.spec.rs`: Unsafe submodule specs and View type.
