// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// TimerTicks Proofs.
// This file contains proof lemmas for the TimerTicks type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================

impl TimerTicks {
    /// Lemma: A newly created TimerTicks has zero ticks.
    pub proof fn lemma_new_is_zero()
        ensures
            TimerTicks::spec_new_view() == (TimerTicksView { ticks: 0 }),
    {
    }

    /// Lemma: MINOR_MODULUS equals u32::MAX + 1.
    pub proof fn lemma_minor_modulus_eq()
        ensures
            TimerTicks::MINOR_MODULUS() == u32::MAX as nat + 1,
            TimerTicks::MINOR_MODULUS() > 0,
    {
    }

    /// Lemma: Any TimerTicks is well-formed (ticks <= u64::MAX).
    ///
    /// # Description
    ///
    /// Since major and minor are both u32, the maximum tick count is
    /// u32::MAX * (u32::MAX + 1) + u32::MAX = u64::MAX.
    pub proof fn lemma_always_wf(&self)
        ensures
            self.wf(),
    {
        assert(self.spec_major() <= u32::MAX as nat);
        assert(self.spec_minor() <= u32::MAX as nat);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: The maximum tick value identity.
    pub proof fn lemma_max_ticks_value()
        ensures
            u32::MAX as nat * TimerTicks::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat,
    {
    }

    /// Lemma: View reflects the tick count.
    pub proof fn lemma_view_reflects_ticks(&self)
        ensures
            self@.ticks == self.spec_ticks(),
    {
    }

    /// Lemma: Two TimerTicks with equal views have equal tick counts.
    pub proof fn lemma_view_equality(a: &TimerTicks, b: &TimerTicks)
        requires
            a@ == b@,
        ensures
            a.spec_ticks() == b.spec_ticks(),
    {
    }

    /// Lemma: A zero-tick counter has both halves zero.
    pub proof fn lemma_zero_implies_both_zero(&self)
        requires
            self.spec_ticks() == 0,
        ensures
            self.spec_major() == 0,
            self.spec_minor() == 0,
    {
        assert(Self::MINOR_MODULUS() > 0);
    }
}

//==================================================================================================
// Proof Lemmas — Increment Properties
//==================================================================================================

impl TimerTicks {
    /// Lemma: When minor < u32::MAX, incrementing minor increases ticks by 1.
    pub proof fn lemma_increment_no_minor_overflow(pre: &TimerTicks)
        requires
            pre.minor < u32::MAX,
        ensures
            ({
                let post = TimerTicks { minor: (pre.minor + 1) as u32, major: pre.major };
                &&& post.spec_ticks() == pre.spec_ticks() + 1
                &&& !pre.spec_is_max()
            }),
    {
        assert(pre.spec_ticks() <= u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat - 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: When minor == u32::MAX and major < u32::MAX, rollover increases ticks by 1.
    pub proof fn lemma_increment_minor_overflow(pre: &TimerTicks)
        requires
            pre.minor == u32::MAX,
            pre.major < u32::MAX,
        ensures
            ({
                let post = TimerTicks { minor: 0u32, major: (pre.major + 1) as u32 };
                &&& post.spec_ticks() == pre.spec_ticks() + 1
                &&& !pre.spec_is_max()
            }),
    {
        assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: When both are u32::MAX, wrapping resets to 0.
    pub proof fn lemma_increment_full_overflow(pre: &TimerTicks)
        requires
            pre.minor == u32::MAX,
            pre.major == u32::MAX,
        ensures
            ({
                let post = TimerTicks { minor: 0u32, major: 0u32 };
                &&& post.spec_ticks() == 0
                &&& pre.spec_is_max()
            }),
    {
        assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: Increment is monotone for non-max counters.
    pub proof fn lemma_increment_monotone(pre: &TimerTicks, post: &TimerTicks)
        requires
            !pre.spec_is_max(),
            pre.wf(),
            if pre.minor < u32::MAX {
                post.minor == (pre.minor + 1) as u32 && post.major == pre.major
            } else {
                post.minor == 0u32 && post.major == (pre.major + 1) as u32
            },
        ensures
            post.spec_ticks() == pre.spec_ticks() + 1,
    {
        if pre.minor < u32::MAX {
            Self::lemma_increment_no_minor_overflow(pre);
        } else {
            assert(pre.minor == u32::MAX);
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
            assert(pre.spec_ticks() == pre.spec_major() * Self::MINOR_MODULUS() + u32::MAX as nat);
            assert(pre.spec_ticks() < u64::MAX as nat);
            assert(pre.major < u32::MAX);
            Self::lemma_increment_minor_overflow(pre);
        }
    }
}

//==================================================================================================
// Proof Lemmas — Ticks Combination
//==================================================================================================

impl TimerTicks {
    /// Lemma: Multiplication is monotone for natural numbers.
    ///
    /// # Description
    ///
    /// If a <= b, then a * c <= b * c. This is used to bound
    /// spec_major() * MINOR_MODULUS() by u32::MAX * MINOR_MODULUS().
    pub proof fn lemma_nat_mul_le_mono(a: nat, b: nat, c: nat)
        requires
            a <= b,
        ensures
            a * c <= b * c,
    {
        assert(a * c <= b * c) by(nonlinear_arith)
            requires(a <= b);
    }

    /// Lemma: The ticks computation doesn't overflow u64.
    pub proof fn lemma_ticks_no_overflow(major: u32, minor: u32)
        ensures
            (major as u64 as nat) * 0x1_0000_0000nat + (minor as u64 as nat) <= u64::MAX as nat,
    {
        assert(u32::MAX as nat * TimerTicks::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: A satisfied wf with non-max state implies ticks < u64::MAX.
    pub proof fn lemma_not_max_implies_lt(&self)
        requires
            self.wf(),
            !self.spec_is_max(),
        ensures
            self.spec_ticks() < u64::MAX as nat,
    {
    }
}

//==================================================================================================
// Proof Lemmas — Monotonicity and Safety
//==================================================================================================

impl TimerTicks {
    /// Lemma: Satisfaction is monotone — once max, wrapping is the only transition.
    pub proof fn lemma_max_is_terminal(pre: &TimerTicks)
        requires
            pre.spec_is_max(),
        ensures
            pre.minor == u32::MAX,
            pre.major == u32::MAX,
    {
        assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        assert(pre.spec_ticks() == pre.spec_major() * Self::MINOR_MODULUS() + pre.spec_minor());
        // If major < u32::MAX, then major * M <= (u32::MAX - 1) * M,
        // and ticks <= (u32::MAX - 1) * M + u32::MAX < u32::MAX * M + u32::MAX = u64::MAX.
        // If major == u32::MAX and minor < u32::MAX, then ticks < u32::MAX * M + u32::MAX = u64::MAX.
        // So both must be u32::MAX.
    }

    /// Lemma: n increments from zero yields ticks == n (when n <= u64::MAX).
    pub proof fn lemma_n_increments_from_zero(n: nat)
        requires
            n <= u64::MAX as nat,
        ensures
            ({
                // After n increments: minor = n % 2^32, major = n / 2^32.
                let minor: nat = n % TimerTicks::MINOR_MODULUS();
                let major: nat = n / TimerTicks::MINOR_MODULUS();
                major * TimerTicks::MINOR_MODULUS() + minor == n
            }),
    {
    }
}

} // verus!
