// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # TimerTicks Implementation
//!
//! A 64-bit tick counter split into two 32-bit halves (major, minor).
//!
//! ## Verified Properties
//!
//! - A new counter starts at zero ticks.
//! - Any (u32, u32) pair is well-formed (ticks <= u64::MAX).
//! - `increment()` increases ticks by exactly 1, or wraps from u64::MAX to 0.
//! - `get()` returns a consistent (major, minor) snapshot tied to `spec_ticks()`.
//! - `ticks()` correctly combines major and minor into a u64.
//! - Monotonicity: each non-wrapping increment increases the tick count.
//! - **`now()` end-to-end safety**: The composed `now()` function proves:
//!   - Nanoseconds < `NANOSECONDS_PER_SECOND` (SystemTime::new() precondition).
//!   - Seconds == `ticks() / timer_freq` (consistency with tick count).
//!   - No u32 overflow in the nanosecond computation.
//!   - `SystemTime::new()` always returns `Some` (the `unreachable!()` is dead code).
//! - **Seconds monotonicity**: If ticks increases, seconds does not decrease.
//! - **`timer_handler_model()`**: Exec-level model proving the handler calls
//!   `increment()` exactly once and its postconditions match `increment()`'s.
//! - **Standalone function models**: `standalone_ticks()` and `standalone_now()`
//!   mirror the original public APIs with full specifications.
//! - **`wrapping_add` equivalence**: `lemma_wrapping_add_equiv` proves that the
//!   explicit branching in `increment()` computes the same result as the
//!   original `wrapping_add(1)` (Trust Boundary T2).
//! - **Torn-read consequence**: `lemma_torn_read_consequence_x86` quantifies the
//!   x86-realistic torn-read error (reader behind by `MINOR_MODULUS` ticks), and
//!   `lemma_torn_read_consequence` covers the weak-memory theoretical case
//!   (reader ahead by `MINOR_MODULUS` ticks). Both are Trust Boundary T1.
//! - **Full `now()` control-flow coverage**: `lemma_now_fallback_valid` and
//!   `lemma_now_pit_valid` prove that both the PIT and fallback `timer_freq`
//!   paths produce valid results. `lemma_now_always_valid` is the top-level
//!   correctness lemma covering any `timer_freq > 0`.
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicU32` for the
//! major and minor counters with a single-writer assumption. For verification,
//! we model these as plain `u32` fields and use `&mut self` for state
//! transitions. This is a sequential model that verifies the counter
//! arithmetic without reasoning about atomicity.
//!
//! **This verified code is a specification model, not a runtime replacement.**
//! The kernel uses the original `src/kernel/src/pm/clock.rs` (with `AtomicU32`
//! and `wrapping_add`) at runtime.
//!
//! ## Verification Scope
//!
//! This verification proves **counter arithmetic correctness**:
//! - The split (major, minor) representation correctly models a 64-bit counter.
//! - Wrapping behavior at u64::MAX is handled correctly.
//! - The `ticks()` combination does not overflow.
//! - The `now()` nanosecond computation is safe and satisfies
//!   `SystemTime::new()`'s precondition (`nanoseconds < NANOSECONDS_PER_SECOND`).
//!
//! Explicitly **out of scope**:
//! - **Concurrency**: The sequential model does not capture concurrent access.
//! - **SystemTime construction**: `SystemTime::new()` is external; we model
//!   only its precondition (`nanoseconds < NANOSECONDS_PER_SECOND`).
//! - **Platform-specific timer frequency**: The actual `timer_freq` value is
//!   determined at runtime; we prove safety for all `timer_freq > 0`.
//!
//! ## API Divergence
//!
//! **`&self` → `&mut self` for `increment()`:** The original `increment(&self)`
//! uses `AtomicU32` interior mutability, allowing shared-reference access. The
//! verified model uses `&mut self` because Verus requires exclusive references
//! for state mutation. This `&mut self` requirement is strictly stronger than
//! the original's `&self` + single-writer assumption. The verified model does
//! **not** prove absence of data races — it proves sequential arithmetic
//! correctness under the assumption that only one writer exists (the timer
//! interrupt handler on a single core).
//!
//! **Global singleton not modeled:** The original code uses a `static TIMER_TICKS`
//! global variable. The verified model operates on arbitrary `TimerTicks`
//! instances, not the global singleton. The singleton access pattern is trusted.
//!
//! **`pub` fields:** Fields `minor` and `major` are `pub` in the verified version
//! (required for Verus `pub open spec fn` access) but private in the original.
//! Since `wf()` is universally true (`lemma_always_wf`), this does not introduce
//! unsoundness, but it weakens encapsulation. The original enforces that only
//! `new()` and `increment()` create/modify `TimerTicks` values.
//!
//! ## Trust Boundaries
//!
//! - **T1: AtomicU32 → plain u32 (Snapshot Consistency).** Atomics are modeled
//!   as plain fields under the single-writer assumption. The original `get()`
//!   loads `major` first, then `minor`. If a timer interrupt fires between the
//!   two loads and increments across a minor-wrap boundary (from `(M, 0xFFFFFFFF)`
//!   to `(M+1, 0)`), the reader observes a **torn read**:
//!
//!   - **x86-realistic** (load-load ordered): reader sees `(M, 0)` — old major,
//!     new minor — which is `MINOR_MODULUS` ticks *behind* the post-increment
//!     state (see `lemma_torn_read_consequence_x86`).
//!   - **Weak-memory theoretical**: reader sees `(M+1, 0xFFFFFFFF)` — new major,
//!     old minor — which is `MINOR_MODULUS` ticks *ahead* of the pre-increment
//!     state (see `lemma_torn_read_consequence`). This cannot occur on x86-TSO.
//!
//!   This assumption is formalized as the opaque uninterpreted spec predicate
//!   `spec_no_concurrent_writer_assumption()`, which is a **precondition** on
//!   `get()`, `now()`, `standalone_ticks()`, and `standalone_now()`. Callers
//!   must obtain this predicate by invoking the `external_body` axiom
//!   `axiom_no_concurrent_writer()` — the axiom is the sole entry point for
//!   the assumption. Because the spec is uninterpreted, Z3 cannot unfold it,
//!   and because it is a `requires` (not unconditionally granted), callers must
//!   explicitly establish the assumption before reading the counter. The trust
//!   boundary assumptions are documented as A-T1a and A-T1b in
//!   `spec_get_consistent`.
//!
//! - **T2: `wrapping_add(1)` → explicit branching.** The original uses
//!   `minor.wrapping_add(1)` which computes `(minor + 1) % 2^32`. The verified
//!   model uses `if minor < u32::MAX { minor + 1 } else { 0 }`, which is
//!   structurally different but semantically identical. `lemma_wrapping_add_equiv`
//!   proves the equivalence: both compute `spec_wrapping_add_one(x)`.
//! - **T3: `timer_handler()`.** This function is **trusted glue code**: it calls
//!   `increment()` exactly once per timer interrupt and does not modify
//!   `major`/`minor` through any other path. Its HAL dependencies
//!   (`InterruptNumber`, `ProcessManager`, platform-specific `#[cfg]` blocks)
//!   are external to the verification. The verified `increment()` contract
//!   establishes what each call achieves, but the handler-level invariant
//!   (single-call-per-interrupt) is assumed, not proved.
//! - **T4: `SystemTime::new()`.** Modeled only via its precondition
//!   (`nanoseconds < NANOSECONDS_PER_SECOND`). The actual SystemTime type is
//!   external.
//! - **T5: Timer frequency.** The `timer_freq > 0` precondition is justified by
//!   platform invariants: the PIT timer frequency is always positive (hardware
//!   guarantee, modeled via `axiom_pit_timer_freq_valid` which returns a ghost
//!   value with `ensures freq > 0`, modeling `pit::get_timer_frequency()`), and
//!   the non-PIT fallback is the compile-time constant `1` (see
//!   `axiom_fallback_timer_freq_valid`). We prove safety for all
//!   `timer_freq > 0` rather than for specific values.

use vstd::prelude::*;

// Include specifications.
include!("clock.spec.rs");

// Include proofs.
include!("clock.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A 64-bit tick counter split into two 32-bit halves.
///
/// # Description
///
/// TimerTicks counts the number of timer interrupts since system startup.
/// The counter is split into major (high 32 bits) and minor (low 32 bits)
/// to model the original atomic implementation.
///
/// # Representation
///
/// Fields are `pub` for Verus spec reasoning (required by `pub open spec fn`).
/// The original uses `AtomicU32` with private fields. External construction of
/// arbitrary `TimerTicks` values is possible in the verified model but not in
/// the original. Since `wf()` is universally true (see `lemma_always_wf`),
/// this does not introduce unsoundness.
pub struct TimerTicks {
    /// Low 32 bits of the tick counter.
    pub minor: u32,
    /// High 32 bits of the tick counter.
    pub major: u32,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl TimerTicks {
    /// Creates a new TimerTicks counter initialized to zero.
    ///
    /// # Description
    ///
    /// Models the original `const fn new()` which creates AtomicU32 pairs.
    ///
    /// # Exec Divergence
    ///
    /// The original is `const fn` and uses `AtomicU32::new(0)`. This model
    /// uses plain `u32` fields (see struct-level documentation for the
    /// AtomicU32 → u32 modeling rationale).
    ///
    /// # Returns
    ///
    /// A new TimerTicks with both halves set to zero.
    pub fn new() -> (result: Self)
        ensures
            result@.ticks == 0,
            result@.is_zero(),
            result@ == TimerTicks::spec_new_view(),
            result.wf(),
    {
        let r = TimerTicks { minor: 0, major: 0 };
        proof { r.lemma_always_wf(); }
        r
    }

    /// Returns the current (major, minor) tick counts.
    ///
    /// # Description
    ///
    /// Models the original `get()` which loads from AtomicU32.
    ///
    /// # Exec Divergence
    ///
    /// The original uses `self.major.load(ORDER)` and `self.minor.load(ORDER)`
    /// (two separate atomic loads). This model accesses plain `u32` fields
    /// directly. Consistency is guaranteed by the single-writer assumption
    /// (Trust Boundary T1), mechanically enforced via the
    /// `spec_no_concurrent_writer_assumption()` precondition.
    ///
    /// # Returns
    ///
    /// A tuple of (major, minor) tick counts.
    pub fn get(&self) -> (result: (u32, u32))
        requires
            self.wf(),
            // Trust Boundary T1: the caller must establish that no concurrent
            // writer can modify major/minor between the two reads.
            Self::spec_no_concurrent_writer_assumption(),
        ensures
            self.spec_get_consistent(result.0, result.1),
            Self::spec_no_concurrent_writer_assumption(),
    {
        (self.major, self.minor)
    }

    /// Increments the tick counter by one.
    ///
    /// # Description
    ///
    /// Models `wrapping_add(1)` on the minor counter. When the minor counter
    /// wraps to 0, the major counter is also incremented (wrapping if at max).
    ///
    /// # Exec Divergence
    ///
    /// - **`&self` → `&mut self`**: The original uses `&self` with `AtomicU32`
    ///   interior mutability. This model uses `&mut self` because Verus requires
    ///   exclusive references for state mutation. The `&mut self` requirement is
    ///   strictly stronger than `&self` + single-writer.
    /// - **`wrapping_add(1)` → explicit branching**: The original uses
    ///   `minor.wrapping_add(1)`. This model uses `if minor < u32::MAX { minor + 1 }
    ///   else { 0 }`, which is semantically identical (Trust Boundary T2, proved
    ///   by `lemma_wrapping_add_equiv`).
    ///
    /// # Returns
    ///
    /// The new minor tick value after incrementing.
    pub fn increment(&mut self) -> (result: u32)
        requires
            // Note: wf() is universally true for any (u32, u32) pair (see
            // lemma_always_wf). This precondition is retained for documentation
            // and forward-compatibility if wf() is ever strengthened.
            old(self).wf(),
        ensures
            self.wf(),
            old(self)@.is_max() ==> self@.ticks == 0,
            !old(self)@.is_max() ==> self@.ticks == old(self)@.ticks + 1,
            self@.ticks == old(self)@.next_ticks(),
    {
        if self.minor < u32::MAX {
            self.minor = self.minor + 1;
            proof {
                assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                assert(old(self).spec_minor() < u32::MAX as nat);
                assert(old(self).spec_major() <= u32::MAX as nat);
                Self::lemma_nat_mul_le_mono(
                    old(self).spec_major(), u32::MAX as nat, Self::MINOR_MODULUS(),
                );
                assert(!old(self).spec_is_max());
                // self.major unchanged, self.minor = old(self).minor + 1.
                assert(self.spec_major() == old(self).spec_major());
                assert(self.spec_minor() == old(self).spec_minor() + 1);
                // spec_ticks = major * M + minor = old.major * M + (old.minor + 1).
                assert(self.spec_ticks() == old(self).spec_major() * Self::MINOR_MODULUS() + old(self).spec_minor() + 1);
                self.lemma_always_wf();
            }
        } else {
            self.minor = 0;
            if self.major < u32::MAX {
                self.major = self.major + 1;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_major() < u32::MAX as nat);
                    Self::lemma_nat_mul_le_mono(
                        old(self).spec_major(), (u32::MAX - 1) as nat, Self::MINOR_MODULUS(),
                    );
                    assert(old(self).spec_ticks() < u64::MAX as nat);
                    assert(!old(self).spec_is_max());
                    // self.major = old.major + 1, self.minor = 0.
                    let om: nat = old(self).spec_major();
                    let m: nat = Self::MINOR_MODULUS();
                    assert((om + 1) * m == om * m + m) by(nonlinear_arith);
                    self.lemma_always_wf();
                }
            } else {
                self.major = 0;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_ticks() == u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat);
                    assert(old(self).spec_is_max());
                    assert(self.spec_ticks() == 0);
                    assert(self.spec_ticks() == old(self).spec_next_ticks());
                    self.lemma_always_wf();
                }
            }
        }
        self.minor
    }

    /// Returns the combined 64-bit tick count.
    ///
    /// # Description
    ///
    /// Combines major and minor into a single u64: `major * 2^32 + minor`.
    /// Models the original standalone `ticks()` function.
    ///
    /// # Exec Divergence
    ///
    /// - The original uses `((major as u64) << 32) + (minor as u64)`. This model
    ///   uses `(major as u64) * 0x1_0000_0000u64 + (minor as u64)`, which is
    ///   semantically identical (`x << 32 == x * 2^32`). Multiplication is used
    ///   because Verus has stronger reasoning support for integer arithmetic than
    ///   for bitwise shift operations.
    /// - The original is a standalone function that calls `TIMER_TICKS.get()`;
    ///   this is a method that accesses fields directly. The standalone pattern
    ///   is mirrored by `standalone_ticks()`.
    ///
    /// # Returns
    ///
    /// The combined tick count as u64.
    pub fn ticks(&self) -> (result: u64)
        requires
            self.wf(),
        ensures
            result as nat == self@.ticks,
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        (self.major as u64) * 0x1_0000_0000u64 + (self.minor as u64)
    }

    /// Checks if the counter is at the maximum value.
    ///
    /// # Description
    ///
    /// Verification-only helper; not present in the original source.
    ///
    /// # Returns
    ///
    /// `true` if ticks == u64::MAX, `false` otherwise.
    pub fn is_max(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.is_max(),
    {
        proof {
            Self::lemma_max_ticks_value();
        }
        self.minor == u32::MAX && self.major == u32::MAX
    }

    /// Checks if the counter is zero.
    ///
    /// # Description
    ///
    /// Verification-only helper; not present in the original source.
    ///
    /// # Returns
    ///
    /// `true` if ticks == 0, `false` otherwise.
    pub fn is_zero(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.is_zero(),
    {
        proof {
            assert(Self::MINOR_MODULUS() > 0);
            if self.major > 0 {
                assert(self.spec_major() >= 1);
                assert(self.spec_major() * Self::MINOR_MODULUS() >= Self::MINOR_MODULUS());
                assert(self.spec_ticks() >= Self::MINOR_MODULUS());
            }
        }
        self.minor == 0 && self.major == 0
    }

    /// Computes the nanosecond component of the current time.
    ///
    /// # Description
    ///
    /// Models the arithmetic from the original `now()` function:
    /// `(minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq)`.
    ///
    /// This function proves that the computation:
    /// - Does not overflow `u32`.
    /// - Produces a result strictly less than `NANOSECONDS_PER_SECOND` (1,000,000,000),
    ///   which satisfies the precondition of `SystemTime::new()`.
    ///
    /// # Parameters
    ///
    /// - `minor_ticks`: The current minor tick count.
    /// - `timer_freq`: The timer frequency in Hz. Must be > 0.
    ///
    /// # Returns
    ///
    /// The nanosecond component, guaranteed < `NANOSECONDS_PER_SECOND`.
    pub fn compute_nanoseconds(minor_ticks: u32, timer_freq: u32) -> (result: u32)
        requires
            timer_freq > 0,
        ensures
            result as nat == Self::spec_compute_nanoseconds(minor_ticks, timer_freq),
            Self::spec_nanoseconds_valid(result as nat),
            result < 1_000_000_000u32,
    {
        proof {
            Self::lemma_nanoseconds_in_range(minor_ticks, timer_freq);
        }
        (minor_ticks % timer_freq) * (1_000_000_000u32 / timer_freq)
    }

    /// Computes the seconds component of the current time.
    ///
    /// # Description
    ///
    /// Models the seconds computation from the original `now()` function:
    /// `(((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64)`.
    ///
    /// # Parameters
    ///
    /// - `major_ticks`: The current major tick count.
    /// - `minor_ticks`: The current minor tick count.
    /// - `timer_freq`: The timer frequency in Hz. Must be > 0.
    ///
    /// # Returns
    ///
    /// The seconds component.
    pub fn compute_seconds(major_ticks: u32, minor_ticks: u32, timer_freq: u32) -> (result: u64)
        requires
            timer_freq > 0,
        ensures
            result as nat == Self::spec_compute_seconds(major_ticks, minor_ticks, timer_freq),
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        let total_ticks: u64 = (major_ticks as u64) * 0x1_0000_0000u64 + (minor_ticks as u64);
        total_ticks / (timer_freq as u64)
    }

    /// Computes the (seconds, nanoseconds) pair for the current time.
    ///
    /// # Description
    ///
    /// This is the verified model of the original standalone `now()` function.
    /// It reads the current tick count via `get()`, computes the seconds and
    /// nanoseconds components, and proves that:
    /// - The nanoseconds component < NANOSECONDS_PER_SECOND (SystemTime precondition).
    /// - The seconds component equals `ticks() / timer_freq`.
    /// - No arithmetic overflow occurs.
    ///
    /// The original `now()` calls `SystemTime::new(seconds, nanoseconds)` which
    /// returns `None` only if `nanoseconds >= NANOSECONDS_PER_SECOND`. This
    /// function proves that can never happen, eliminating the `unreachable!()`
    /// panic path.
    ///
    /// # Exec Divergence
    ///
    /// - **Return type**: The original returns `SystemTime`; this returns
    ///   `(u64, u32)` because `SystemTime` is an external type not available
    ///   in the Verus model. The pair represents `(seconds, nanoseconds)`.
    /// - **`timer_freq` parameter**: The original determines `timer_freq` via
    ///   `#[cfg(feature = "pit")]` branches; this takes it as a parameter
    ///   because Verus cannot model `#[cfg]` conditional compilation. The
    ///   `now_fallback_model()` and `now_pit_model()` functions cover both
    ///   cfg paths.
    /// - **Helper decomposition**: The inline arithmetic is factored into
    ///   `compute_seconds()` and `compute_nanoseconds()` for modular proof
    ///   decomposition. The arithmetic is identical to the original.
    /// - **No `unreachable!()`**: The proof eliminates the panic path by
    ///   showing `nanoseconds < NANOSECONDS_PER_SECOND` always holds.
    ///
    /// # Parameters
    ///
    /// - `timer_freq`: The timer frequency in Hz. Must be > 0. In the original,
    ///   this is `pit::get_timer_frequency()` or the fallback value 1.
    ///
    /// # Returns
    ///
    /// A (seconds, nanoseconds) pair where `nanoseconds < 1_000_000_000`.
    pub fn now(&self, timer_freq: u32) -> (result: (u64, u32))
        requires
            timer_freq > 0,
            self.wf(),
            Self::spec_no_concurrent_writer_assumption(),
        ensures
            result.0 as nat == self@.ticks / timer_freq as nat,
            result.1 as nat == self@.nanoseconds(timer_freq as nat),
            Self::spec_nanoseconds_valid(result.1 as nat),
            result.1 < 1_000_000_000u32,
    {
        let (major_ticks, minor_ticks): (u32, u32) = self.get();
        let seconds: u64 = Self::compute_seconds(major_ticks, minor_ticks, timer_freq);
        let nanoseconds: u32 = Self::compute_nanoseconds(minor_ticks, timer_freq);
        proof {
            // Prove minor_ticks == self@.ticks % MINOR_MODULUS (for nanoseconds equivalence).
            let m: nat = Self::MINOR_MODULUS();
            let minor_nat: nat = minor_ticks as nat;
            let major_nat: nat = major_ticks as nat;
            assert(minor_nat < m);
            assert(major_nat * m + minor_nat == self.spec_ticks());
            assert((major_nat * m + minor_nat) % m == minor_nat) by(nonlinear_arith)
                requires(minor_nat < m && m > 0);
        }
        (seconds, nanoseconds)
    }

    //==============================================================================================
    // Exec-Level timer_handler Model
    //==============================================================================================

    /// Models the timer_handler() function at exec level.
    ///
    /// # Description
    ///
    /// This is the verified behavioral model of the original `timer_handler()`.
    /// The original `timer_handler()` cannot be directly ported to Verus because
    /// it depends on `unsafe`, HAL types (`InterruptNumber`), global mutable
    /// state (`TIMER_TICKS`), `#[cfg]` conditional compilation, and scheduler
    /// interactions (`ProcessManager::giveup()`).
    ///
    /// This model calls `increment()` exactly once, which is the only effect of
    /// the handler on the clock counter. The original handler also:
    /// - Checks for VM pause requests (volatile read + I/O port write).
    /// - Attempts a context switch via `ProcessManager::giveup()`.
    ///
    /// These side effects are HAL/scheduler interactions modeled as Trust
    /// Boundary T3: they do not modify `major` or `minor`.
    ///
    /// # Exec Divergence
    ///
    /// - **Name**: `timer_handler_model` (not `timer_handler`) to distinguish
    ///   the verified model from the original.
    /// - **Signature**: The original is `pub unsafe fn timer_handler(_intnum:
    ///   InterruptNumber)` accessing the global `TIMER_TICKS`. This model takes
    ///   `&mut self` and operates on a local instance.
    /// - **Side effects omitted**: VM pause check and context switch are
    ///   HAL/scheduler interactions that do not affect clock state (T3).
    pub fn timer_handler_model(&mut self)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            self@.ticks == old(self)@.next_ticks(),
            old(self)@.is_max() ==> self@.ticks == 0,
            !old(self)@.is_max() ==> self@.ticks == old(self)@.ticks + 1,
    {
        self.increment();
    }
}

//==================================================================================================
// Standalone Function Models
//==================================================================================================

/// Standalone model of the original `pub fn ticks() -> u64`.
///
/// # Description
///
/// The original `ticks()` reads from the global `TIMER_TICKS` singleton via
/// `TIMER_TICKS.get()`, which performs two separate atomic loads. This model
/// mirrors that by calling `get()` and combining the result, requiring
/// `spec_no_concurrent_writer_assumption()` to match the original's implicit
/// dependency on snapshot consistency (Trust Boundary T1).
pub fn standalone_ticks(timer: &TimerTicks) -> (result: u64)
    requires
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result as nat == timer@.ticks,
{
    let (major, minor): (u32, u32) = timer.get();
    proof {
        assert(major <= u32::MAX);
        assert(minor <= u32::MAX);
        assert(u32::MAX as nat * TimerTicks::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        TimerTicks::lemma_nat_mul_le_mono(major as nat, u32::MAX as nat, TimerTicks::MINOR_MODULUS());
    }
    (major as u64) * 0x1_0000_0000u64 + (minor as u64)
}

/// Standalone model of the original `pub fn now() -> SystemTime`.
///
/// # Description
///
/// The original `now()` reads from the global `TIMER_TICKS` singleton,
/// determines `timer_freq` from a `#[cfg]` branch, computes (seconds,
/// nanoseconds), and constructs a `SystemTime`. This model takes
/// explicit `&TimerTicks` and `timer_freq` parameters and returns a
/// `(u64, u32)` pair representing (seconds, nanoseconds).
///
/// The postconditions prove:
/// - `nanoseconds < NANOSECONDS_PER_SECOND` (SystemTime::new() precondition).
/// - `seconds == ticks / timer_freq` (consistency).
/// - The pair matches `spec_now()`.
///
/// The `SystemTime::new()` call in the original is modeled via
/// `spec_system_time_new_succeeds`, which is proved to hold by
/// `lemma_now_valid_for_system_time`. Since `nanoseconds < NANOSECONDS_PER_SECOND`
/// is guaranteed by the postcondition, `SystemTime::new()` always returns `Some`,
/// making the `unreachable!()` branch in the original `now()` dead code.
pub fn standalone_now(timer: &TimerTicks, timer_freq: u32) -> (result: (u64, u32))
    requires
        timer_freq > 0,
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result.0 as nat == timer@.ticks / timer_freq as nat,
        result.1 as nat == timer@.nanoseconds(timer_freq as nat),
        TimerTicks::spec_nanoseconds_valid(result.1 as nat),
        result.1 < 1_000_000_000u32,
        TimerTicks::spec_system_time_new_succeeds(result.1 as nat),
{
    timer.now(timer_freq)
}

/// Exec-level model of the original `now()` for the fallback path.
///
/// # Description
///
/// Models the `#[cfg(not(feature = "pit"))]` branch of the original `now()`:
/// ```ignore
/// let timer_freq: u32 = 1;
/// let (major_ticks, minor_ticks) = TIMER_TICKS.get();
/// let seconds = (((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64);
/// let nanoseconds = (minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq);
/// match SystemTime::new(seconds, nanoseconds) { Some(t) => t, None => unreachable!() }
/// ```
///
/// This function hardcodes `timer_freq = 1` (the compile-time constant) and
/// proves all postconditions including `SystemTime::new()` success, eliminating
/// the `unreachable!()` path. The `SystemTime::new()` call is modeled via
/// `spec_system_time_new_succeeds`.
pub fn now_fallback_model(timer: &TimerTicks) -> (result: (u64, u32))
    requires
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result.0 as nat == timer@.ticks / 1nat,
        result.1 as nat == timer@.nanoseconds(1nat),
        TimerTicks::spec_nanoseconds_valid(result.1 as nat),
        result.1 < 1_000_000_000u32,
        TimerTicks::spec_system_time_new_succeeds(result.1 as nat),
{
    // #[cfg(not(feature = "pit"))]
    let timer_freq: u32 = 1u32;
    timer.now(timer_freq)
}

/// Exec-level model of the original `now()` for the PIT path.
///
/// # Description
///
/// Models the `#[cfg(feature = "pit")]` branch of the original `now()`:
/// ```ignore
/// let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
/// ```
///
/// Since `pit::get_timer_frequency()` is a HAL function that Verus cannot
/// call, this model takes `timer_freq` as a parameter with the precondition
/// `timer_freq > 0` — the same guarantee provided by
/// `axiom_pit_timer_freq_valid()`. A caller would:
/// 1. Invoke `axiom_pit_timer_freq_valid()` to obtain a ghost `freq > 0`.
/// 2. Call this function with the actual PIT frequency at runtime.
///
/// The postconditions prove `SystemTime::new()` success for any PIT frequency.
pub fn now_pit_model(timer: &TimerTicks, timer_freq: u32) -> (result: (u64, u32))
    requires
        timer_freq > 0,
        timer.wf(),
        TimerTicks::spec_no_concurrent_writer_assumption(),
    ensures
        result.0 as nat == timer@.ticks / timer_freq as nat,
        result.1 as nat == timer@.nanoseconds(timer_freq as nat),
        TimerTicks::spec_nanoseconds_valid(result.1 as nat),
        result.1 < 1_000_000_000u32,
        TimerTicks::spec_system_time_new_succeeds(result.1 as nat),
{
    // #[cfg(feature = "pit")]
    // let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
    timer.now(timer_freq)
}

} // verus!
