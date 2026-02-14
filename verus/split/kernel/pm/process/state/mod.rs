// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

use vstd::prelude::*;

// Include specifications.
include!("mod.spec.rs");

// Include proofs.
include!("mod.proof.rs");

pub mod interrupted;
pub mod process_state;
pub mod runnable;
pub mod running;
pub mod sleeping;
pub mod zombie;
