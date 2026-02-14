// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications for the thread module index.
//
// This module is a re-export index (`pub mod` declarations only) and
// contains no types, functions, or logic requiring specification.
// All verifiable types are specified in their respective submodule
// spec files:
// - ThreadManager, ReadyThread, ThreadRefModel, ThreadRefMutModel
//   → thread_manager.spec.rs
// - ThreadState → state.spec.rs
// - InterruptedThread → interrupted.spec.rs
// - RunningThread → running.spec.rs
// - SleepingThread → sleeping.spec.rs
// - ZombieThread → zombie.spec.rs
