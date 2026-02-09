// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// TimerTicks Specification.
// This file contains spec functions for the TimerTicks type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a TimerTicks counter.
///
/// # Description
///
/// Represents the observable state of the timer as a single combined tick count.
/// The split (major, minor) representation is an implementation detail.
#[verifier::ext_equal]
pub struct TimerTicksView {
    /// Combined 64-bit tick count.
    pub ticks: nat,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl TimerTicks {
    /// Spec function: the modulus for the minor counter (2^32 = u32::MAX + 1).
    ///
    /// # Description
    ///
    /// This is the number of minor ticks per major tick epoch.
    /// Equal to 0x1_0000_0000 = 4294967296.
    pub open spec fn MINOR_MODULUS() -> nat {
        u32::MAX as nat + 1
    }

    /// Spec function: returns the minor tick count.
    pub open spec fn spec_minor(&self) -> nat {
        self.minor as nat
    }

    /// Spec function: returns the major tick count.
    pub open spec fn spec_major(&self) -> nat {
        self.major as nat
    }

    /// Spec function: returns the combined 64-bit tick count.
    ///
    /// # Description
    ///
    /// The abstract tick count is `major * 2^32 + minor`.
    pub open spec fn spec_ticks(&self) -> nat {
        self.spec_major() * Self::MINOR_MODULUS() + self.spec_minor()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// The tick count is representable as a u64. This is always true for
    /// valid (u32, u32) pairs, as proved by `lemma_always_wf`.
    pub open spec fn wf(&self) -> bool {
        self.spec_ticks() <= u64::MAX as nat
    }

    /// Spec function: whether the counter is at its maximum value (u64::MAX).
    pub open spec fn spec_is_max(&self) -> bool {
        self.spec_ticks() == u64::MAX as nat
    }

    /// Spec function: whether the counter is zero.
    pub open spec fn spec_is_zero(&self) -> bool {
        self.spec_ticks() == 0
    }

    /// Spec function: the expected tick count after one increment.
    ///
    /// # Description
    ///
    /// If at max, wraps to 0. Otherwise, increases by 1.
    pub open spec fn spec_next_ticks(&self) -> nat {
        if self.spec_is_max() { 0 } else { self.spec_ticks() + 1 }
    }

    /// Spec function: the view of a newly created TimerTicks.
    pub open spec fn spec_new_view() -> TimerTicksView {
        TimerTicksView { ticks: 0 }
    }

    //==============================================================================================
    // now() Arithmetic Specs
    //==============================================================================================

    /// Constant: nanoseconds per second (1,000,000,000).
    pub open spec fn NANOSECONDS_PER_SECOND() -> nat {
        1_000_000_000
    }

    /// Spec function: checks if a timer frequency is valid (non-zero).
    pub open spec fn spec_timer_freq_valid(timer_freq: u32) -> bool {
        timer_freq > 0
    }

    /// Spec function: computes the nanosecond component of the current time.
    ///
    /// # Description
    ///
    /// Models the original expression:
    /// `(minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq)`.
    pub open spec fn spec_compute_nanoseconds(minor_ticks: u32, timer_freq: u32) -> nat
        recommends timer_freq > 0,
    {
        (minor_ticks as nat % timer_freq as nat) * (Self::NANOSECONDS_PER_SECOND() / timer_freq as nat)
    }

    /// Spec function: computes the seconds component of the current time.
    ///
    /// # Description
    ///
    /// Models the original expression:
    /// `(((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64)`.
    pub open spec fn spec_compute_seconds(major_ticks: u32, minor_ticks: u32, timer_freq: u32) -> nat
        recommends timer_freq > 0,
    {
        let total_ticks: nat = major_ticks as nat * Self::MINOR_MODULUS() + minor_ticks as nat;
        total_ticks / timer_freq as nat
    }

    /// Spec function: checks if nanoseconds are valid for SystemTime::new().
    ///
    /// # Description
    ///
    /// SystemTime::new(seconds, nanoseconds) returns None iff
    /// nanoseconds >= NANOSECONDS_PER_SECOND.
    pub open spec fn spec_nanoseconds_valid(nanoseconds: nat) -> bool {
        nanoseconds < Self::NANOSECONDS_PER_SECOND()
    }

    //==============================================================================================
    // now() Composed Specs
    //==============================================================================================

    /// Spec function: full `now()` result as a (seconds, nanoseconds) pair.
    ///
    /// # Description
    ///
    /// Models the complete `now()` computation from the original source.
    /// Given a TimerTicks state and a timer frequency, computes both the
    /// seconds and nanoseconds components.
    pub open spec fn spec_now(&self, timer_freq: u32) -> (nat, nat)
        recommends timer_freq > 0,
    {
        (
            Self::spec_compute_seconds(self.major, self.minor, timer_freq),
            Self::spec_compute_nanoseconds(self.minor, timer_freq),
        )
    }

    /// Spec function: the seconds component of `now()` is consistent with `ticks()`.
    ///
    /// # Description
    ///
    /// The seconds value equals `spec_ticks() / timer_freq`, which is the
    /// same as `ticks() / timer_freq` at the exec level.
    pub open spec fn spec_seconds_consistent_with_ticks(&self, timer_freq: u32) -> bool
        recommends timer_freq > 0,
    {
        Self::spec_compute_seconds(self.major, self.minor, timer_freq) == self.spec_ticks() / timer_freq as nat
    }

    //==============================================================================================
    // get() Consistency Specs
    //==============================================================================================

    /// Spec function: a (major, minor) pair from `get()` is consistent with `spec_ticks()`.
    ///
    /// # Description
    ///
    /// Given a pair returned by `get()`, the combined tick count equals
    /// `major * MINOR_MODULUS + minor`, which is exactly `spec_ticks()`.
    /// This establishes that the pair is a consistent snapshot.
    ///
    /// # Trust Boundary T1: Atomic Snapshot Consistency
    ///
    /// The original `get()` performs two separate atomic loads:
    /// ```ignore
    /// (self.major.load(ORDER), self.minor.load(ORDER))
    /// ```
    /// Under the **single-writer assumption** (only the timer interrupt handler
    /// on one core modifies the counter), tearing cannot occur because the
    /// writer is not concurrent with the reader (interrupts are serialized on
    /// the same core, and `get()` is called with interrupts disabled or from
    /// the same interrupt context).
    ///
    /// This consistency property is **assumed**, not proved. The assumption is:
    /// - **A-T1a**: No concurrent writer modifies `major` or `minor` between
    ///   the two loads in `get()`.
    /// - **A-T1b**: The memory ordering (`Ordering::Relaxed` with single-writer)
    ///   ensures visibility of the latest write.
    ///
    /// These assumptions hold under Nanvix's architecture: the timer handler
    /// runs in interrupt context on a single core, and `get()` callers either
    /// run on the same core (serialized by interrupt enable/disable) or on
    /// other cores where the atomic visibility guarantee is sufficient.
    pub open spec fn spec_get_consistent(&self, major: u32, minor: u32) -> bool {
        major as nat * Self::MINOR_MODULUS() + minor as nat == self.spec_ticks()
    }

    //==============================================================================================
    // SystemTime Model
    //==============================================================================================

    /// Spec function: models the success condition of `SystemTime::new()`.
    ///
    /// # Description
    ///
    /// `SystemTime::new(seconds, nanoseconds)` returns `Some(...)` iff
    /// `nanoseconds < NANOSECONDS_PER_SECOND`. This spec function captures
    /// that condition, allowing us to prove that the `unreachable!()` path
    /// in the original `now()` is dead code.
    pub open spec fn spec_system_time_new_succeeds(nanoseconds: nat) -> bool {
        nanoseconds < Self::NANOSECONDS_PER_SECOND()
    }

    //==============================================================================================
    // Concurrency / Snapshot Assumption Specs
    //==============================================================================================

    /// Spec function: the wrapping-add result for a u32 value.
    ///
    /// # Description
    ///
    /// Models `x.wrapping_add(1)` at the spec level: `(x + 1) % 2^32`.
    /// This is used to show that our branching `increment()` model computes
    /// the same result as the original `wrapping_add(1)`.
    pub open spec fn spec_wrapping_add_one(x: u32) -> nat {
        (x as nat + 1) % Self::MINOR_MODULUS()
    }

    /// Spec function: formal statement of the no-concurrent-writer assumption.
    ///
    /// # Description
    ///
    /// Models the system-level invariant required for `get()` to return a
    /// consistent (major, minor) snapshot. This assumption is **not proved**
    /// within the clock module — it is a system-level property that depends on:
    ///
    /// 1. **Single-writer**: Only the timer interrupt handler on a single core
    ///    modifies `major` and `minor`.
    /// 2. **Reader serialization**: Callers of `get()` either:
    ///    (a) run on the same core with interrupts disabled, preventing the
    ///        handler from executing between the two loads, or
    ///    (b) accept the single-writer guarantee as sufficient for consistency
    ///        (since the writer is atomic at the word level).
    ///
    /// When this assumption holds, the two separate loads in `get()` observe a
    /// single consistent state. When violated, a **torn read** can occur:
    ///
    /// - **x86-realistic** (`get()` loads major then minor, x86-TSO orders loads):
    ///   reader sees `(M, 0)` — old major, new minor — `MINOR_MODULUS` ticks
    ///   *behind* reality (see `lemma_torn_read_consequence_x86`).
    /// - **Weak-memory theoretical** (load reordering possible): reader sees
    ///   `(M+1, 0xFFFFFFFF)` — new major, old minor — `MINOR_MODULUS` ticks
    ///   *ahead* of reality (see `lemma_torn_read_consequence`).
    ///
    /// This spec function is deliberately **opaque** (not `open`): it cannot
    /// be unfolded by Z3 to `true`, so any proof that depends on snapshot
    /// consistency must explicitly assume or propagate this predicate. This
    /// makes the trust boundary mechanically visible in the proof chain.
    ///
    /// The assumption is introduced into the proof environment via
    /// `axiom_no_concurrent_writer()` in the proof file, which is an
    /// `external_body` axiom. Only code paths that invoke this axiom (or
    /// receive it from `get()`'s postcondition) can rely on consistency.
    pub uninterp spec fn spec_no_concurrent_writer_assumption() -> bool;

    //==============================================================================================
    // timer_handler Behavioral Specs
    //==============================================================================================

    /// Spec function: models the observable effect of one `timer_handler()` call.
    ///
    /// # Description
    ///
    /// The timer handler's only effect on the clock state is to call
    /// `increment()` exactly once. The result state has ticks equal to
    /// `spec_next_ticks()` of the pre-state. The handler's other side effects
    /// (VM pause check via volatile read, context switch via `ProcessManager::giveup()`)
    /// are HAL/scheduler interactions that do not modify the clock counter.
    pub open spec fn spec_timer_handler_effect(&self, post: &TimerTicks) -> bool {
        post.spec_ticks() == self.spec_next_ticks()
    }

    //==============================================================================================
    // Timer Frequency Specs
    //==============================================================================================

    /// Spec function: platform timer frequency guarantee.
    ///
    /// # Description
    ///
    /// On all supported platforms, the timer frequency is positive:
    /// - `#[cfg(feature = "pit")]`: `pit::get_timer_frequency()` returns the PIT
    ///   oscillator frequency divided by the programmed divisor, which is always > 0.
    /// - `#[cfg(not(feature = "pit"))]`: the fallback sets `timer_freq = 1`.
    ///
    /// This spec captures the platform invariant that justifies the
    /// `timer_freq > 0` precondition on `compute_nanoseconds` and
    /// `compute_seconds`.
    pub open spec fn spec_platform_timer_freq_valid(timer_freq: u32) -> bool {
        timer_freq > 0
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for TimerTicks {
    type V = TimerTicksView;

    open spec fn view(&self) -> TimerTicksView {
        TimerTicksView { ticks: self.spec_ticks() }
    }
}

} // verus!
