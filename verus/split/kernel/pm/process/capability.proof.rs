// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Capabilities Proofs.
// This file contains proof lemmas for the Capabilities type.

verus! {

//==================================================================================================
// Proof Lemmas
//==================================================================================================

impl Capabilities {
    /// Lemma: Default capabilities have no bits set.
    pub proof fn lemma_default_is_empty()
        ensures
            Capabilities::spec_default().spec_bits() == 0u8,
            !Capabilities::spec_default().spec_has(Capability::ExceptionControl),
            !Capabilities::spec_default().spec_has(Capability::InterruptControl),
            !Capabilities::spec_default().spec_has(Capability::IoManagement),
            !Capabilities::spec_default().spec_has(Capability::MemoryManagement),
            !Capabilities::spec_default().spec_has(Capability::ProcessManagement),
            Capabilities::spec_default().wf(),
    {
        assert(0u8 & 1u8 == 0u8) by (bit_vector);
        assert(0u8 & 2u8 == 0u8) by (bit_vector);
        assert(0u8 & 4u8 == 0u8) by (bit_vector);
        assert(0u8 & 8u8 == 0u8) by (bit_vector);
        assert(0u8 & 16u8 == 0u8) by (bit_vector);
        assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
    }

    /// Lemma: The spec_mask for each variant is a single distinct bit.
    pub proof fn lemma_masks_distinct()
        ensures
            Capabilities::spec_mask(Capability::ExceptionControl) == 1u8,
            Capabilities::spec_mask(Capability::InterruptControl) == 2u8,
            Capabilities::spec_mask(Capability::IoManagement) == 4u8,
            Capabilities::spec_mask(Capability::MemoryManagement) == 8u8,
            Capabilities::spec_mask(Capability::ProcessManagement) == 16u8,
    {
    }

    /// Lemma: Each explicit mask value equals `2^discriminant`, formally linking
    /// `spec_mask` to the original source's `1 << capability as u8` formula.
    ///
    /// # Description
    ///
    /// The original source computes masks via `1 << capability as u8`, relying
    /// on Rust's enum discriminant assignment. The verified version uses explicit
    /// match-based mapping. This lemma proves their equivalence by showing that
    /// `spec_mask(cap) == spec_pow2_mask(cap.spec_discriminant())` for all variants.
    pub proof fn lemma_mask_matches_discriminant(cap: Capability)
        ensures
            Self::spec_mask(cap) == Self::spec_pow2_mask(cap.spec_discriminant()),
    {
        // Exhaustive case analysis: each variant's mask matches 2^discriminant.
        match cap {
            Capability::ExceptionControl => {},
            Capability::InterruptControl => {},
            Capability::IoManagement => {},
            Capability::MemoryManagement => {},
            Capability::ProcessManagement => {},
        }
    }

    /// Lemma: Setting a capability bit ensures `has` returns true for that capability.
    pub proof fn lemma_set_then_has(pre: Capabilities, cap: Capability)
        ensures
            ({
                let post_bits: u8 = pre.spec_set(cap);
                (post_bits & Self::spec_mask(cap)) != 0u8
            }),
    {
        let b: u8 = pre.spec_bits();
        match cap {
            Capability::ExceptionControl => {
                assert(((b | 1u8) as u8 & 1u8) != 0u8) by (bit_vector);
            },
            Capability::InterruptControl => {
                assert(((b | 2u8) as u8 & 2u8) != 0u8) by (bit_vector);
            },
            Capability::IoManagement => {
                assert(((b | 4u8) as u8 & 4u8) != 0u8) by (bit_vector);
            },
            Capability::MemoryManagement => {
                assert(((b | 8u8) as u8 & 8u8) != 0u8) by (bit_vector);
            },
            Capability::ProcessManagement => {
                assert(((b | 16u8) as u8 & 16u8) != 0u8) by (bit_vector);
            },
        }
    }

    /// Lemma: Clearing a capability bit ensures `has` returns false for that capability.
    pub proof fn lemma_clear_then_not_has(pre: Capabilities, cap: Capability)
        ensures
            ({
                let post_bits: u8 = pre.spec_clear(cap);
                (post_bits & Self::spec_mask(cap)) == 0u8
            }),
    {
        let b: u8 = pre.spec_bits();
        match cap {
            Capability::ExceptionControl => {
                assert(((b & !1u8) as u8 & 1u8) == 0u8) by (bit_vector);
            },
            Capability::InterruptControl => {
                assert(((b & !2u8) as u8 & 2u8) == 0u8) by (bit_vector);
            },
            Capability::IoManagement => {
                assert(((b & !4u8) as u8 & 4u8) == 0u8) by (bit_vector);
            },
            Capability::MemoryManagement => {
                assert(((b & !8u8) as u8 & 8u8) == 0u8) by (bit_vector);
            },
            Capability::ProcessManagement => {
                assert(((b & !16u8) as u8 & 16u8) == 0u8) by (bit_vector);
            },
        }
    }

    /// Lemma: Distinct capabilities have disjoint (non-overlapping) masks.
    pub proof fn lemma_distinct_masks_disjoint(a: Capability, b: Capability)
        requires
            Self::spec_mask(a) != Self::spec_mask(b),
        ensures
            Self::spec_mask(a) & Self::spec_mask(b) == 0u8,
    {
        // Exhaustive case analysis on both capabilities.
        match a {
            Capability::ExceptionControl => { match b {
                Capability::InterruptControl => { assert(1u8 & 2u8 == 0u8) by (bit_vector); },
                Capability::IoManagement => { assert(1u8 & 4u8 == 0u8) by (bit_vector); },
                Capability::MemoryManagement => { assert(1u8 & 8u8 == 0u8) by (bit_vector); },
                Capability::ProcessManagement => { assert(1u8 & 16u8 == 0u8) by (bit_vector); },
                _ => {},
            }},
            Capability::InterruptControl => { match b {
                Capability::ExceptionControl => { assert(2u8 & 1u8 == 0u8) by (bit_vector); },
                Capability::IoManagement => { assert(2u8 & 4u8 == 0u8) by (bit_vector); },
                Capability::MemoryManagement => { assert(2u8 & 8u8 == 0u8) by (bit_vector); },
                Capability::ProcessManagement => { assert(2u8 & 16u8 == 0u8) by (bit_vector); },
                _ => {},
            }},
            Capability::IoManagement => { match b {
                Capability::ExceptionControl => { assert(4u8 & 1u8 == 0u8) by (bit_vector); },
                Capability::InterruptControl => { assert(4u8 & 2u8 == 0u8) by (bit_vector); },
                Capability::MemoryManagement => { assert(4u8 & 8u8 == 0u8) by (bit_vector); },
                Capability::ProcessManagement => { assert(4u8 & 16u8 == 0u8) by (bit_vector); },
                _ => {},
            }},
            Capability::MemoryManagement => { match b {
                Capability::ExceptionControl => { assert(8u8 & 1u8 == 0u8) by (bit_vector); },
                Capability::InterruptControl => { assert(8u8 & 2u8 == 0u8) by (bit_vector); },
                Capability::IoManagement => { assert(8u8 & 4u8 == 0u8) by (bit_vector); },
                Capability::ProcessManagement => { assert(8u8 & 16u8 == 0u8) by (bit_vector); },
                _ => {},
            }},
            Capability::ProcessManagement => { match b {
                Capability::ExceptionControl => { assert(16u8 & 1u8 == 0u8) by (bit_vector); },
                Capability::InterruptControl => { assert(16u8 & 2u8 == 0u8) by (bit_vector); },
                Capability::IoManagement => { assert(16u8 & 4u8 == 0u8) by (bit_vector); },
                Capability::MemoryManagement => { assert(16u8 & 8u8 == 0u8) by (bit_vector); },
                _ => {},
            }},
        }
    }

    /// Lemma: Setting a capability preserves other bits.
    pub proof fn lemma_set_preserves_other(pre: Capabilities, cap_set: Capability, cap_other: Capability)
        requires
            Self::spec_mask(cap_set) != Self::spec_mask(cap_other),
        ensures
            ({
                let post_bits: u8 = pre.spec_set(cap_set);
                ((post_bits & Self::spec_mask(cap_other)) != 0u8)
                ==
                ((pre.spec_bits() & Self::spec_mask(cap_other)) != 0u8)
            }),
    {
        let mask_s: u8 = Self::spec_mask(cap_set);
        let mask_o: u8 = Self::spec_mask(cap_other);
        let b: u8 = pre.spec_bits();

        Self::lemma_distinct_masks_disjoint(cap_set, cap_other);

        assert(((b | mask_s) as u8 & mask_o != 0u8) == (b & mask_o != 0u8)) by (bit_vector)
            requires
                mask_s & mask_o == 0u8,
        ;
    }

    /// Lemma: Clearing a capability preserves other bits.
    pub proof fn lemma_clear_preserves_other(pre: Capabilities, cap_clear: Capability, cap_other: Capability)
        requires
            Self::spec_mask(cap_clear) != Self::spec_mask(cap_other),
        ensures
            ({
                let post_bits: u8 = pre.spec_clear(cap_clear);
                ((post_bits & Self::spec_mask(cap_other)) != 0u8)
                ==
                ((pre.spec_bits() & Self::spec_mask(cap_other)) != 0u8)
            }),
    {
        let mask_c: u8 = Self::spec_mask(cap_clear);
        let mask_o: u8 = Self::spec_mask(cap_other);
        let b: u8 = pre.spec_bits();

        Self::lemma_distinct_masks_disjoint(cap_clear, cap_other);

        assert(((b & !mask_c) as u8 & mask_o != 0u8) == (b & mask_o != 0u8)) by (bit_vector)
            requires
                mask_c & mask_o == 0u8,
        ;
    }

    /// Lemma: Setting an already-set bit is idempotent.
    pub proof fn lemma_set_idempotent(pre: Capabilities, cap: Capability)
        requires
            pre.spec_has(cap),
        ensures
            pre.spec_set(cap) == pre.spec_bits(),
    {
        let mask: u8 = Self::spec_mask(cap);
        let b: u8 = pre.spec_bits();
        assert((b | mask) as u8 == b) by (bit_vector)
            requires
                (b & mask) != 0u8,
                mask == 1u8 || mask == 2u8 || mask == 4u8 || mask == 8u8 || mask == 16u8,
        ;
    }

    /// Lemma: Clearing an already-clear bit is idempotent.
    pub proof fn lemma_clear_idempotent(pre: Capabilities, cap: Capability)
        requires
            !pre.spec_has(cap),
        ensures
            pre.spec_clear(cap) == pre.spec_bits(),
    {
        let mask: u8 = Self::spec_mask(cap);
        let b: u8 = pre.spec_bits();
        assert((b & !mask) as u8 == b) by (bit_vector)
            requires
                (b & mask) == 0u8,
                mask == 1u8 || mask == 2u8 || mask == 4u8 || mask == 8u8 || mask == 16u8,
        ;
    }

    /// Lemma: `set` preserves well-formedness.
    ///
    /// # Description
    ///
    /// If the upper 3 bits (5, 6, 7) are clear before `set`, they remain clear
    /// afterward, because all capability masks (1, 2, 4, 8, 16) only affect bits 0..=4.
    pub proof fn lemma_set_preserves_wf(pre: Capabilities, cap: Capability)
        requires
            pre.wf(),
        ensures
            ({
                let post: Capabilities = Capabilities { bits: pre.spec_set(cap) };
                post.wf()
            }),
    {
        let b: u8 = pre.spec_bits();
        match cap {
            Capability::ExceptionControl => {
                assert(((b | 1u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::InterruptControl => {
                assert(((b | 2u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::IoManagement => {
                assert(((b | 4u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::MemoryManagement => {
                assert(((b | 8u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::ProcessManagement => {
                assert(((b | 16u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
        }
    }

    /// Lemma: `clear` preserves well-formedness.
    ///
    /// # Description
    ///
    /// If the upper 3 bits (5, 6, 7) are clear before `clear`, they remain clear
    /// afterward, because AND-with-complement can only clear bits, never set them.
    pub proof fn lemma_clear_preserves_wf(pre: Capabilities, cap: Capability)
        requires
            pre.wf(),
        ensures
            ({
                let post: Capabilities = Capabilities { bits: pre.spec_clear(cap) };
                post.wf()
            }),
    {
        let b: u8 = pre.spec_bits();
        match cap {
            Capability::ExceptionControl => {
                assert(((b & !1u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::InterruptControl => {
                assert(((b & !2u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::IoManagement => {
                assert(((b & !4u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::MemoryManagement => {
                assert(((b & !8u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
            Capability::ProcessManagement => {
                assert(((b & !16u8) as u8) & 0b1110_0000u8 == 0u8) by (bit_vector)
                    requires b & 0b1110_0000u8 == 0u8;
            },
        }
    }

    /// Lemma: Roundtrip — clearing after setting restores the original bitfield
    /// when the bit was originally clear.
    ///
    /// # Description
    ///
    /// If capability `cap` is not set in `pre`, then `clear(set(pre, cap), cap)`
    /// returns the original bitfield. This proves that `set` and `clear` are
    /// inverses for the grant/revoke pattern.
    pub proof fn lemma_set_clear_roundtrip(pre: Capabilities, cap: Capability)
        requires
            !pre.spec_has(cap),
        ensures
            ({
                let after_set: Capabilities = Capabilities { bits: pre.spec_set(cap) };
                after_set.spec_clear(cap) == pre.spec_bits()
            }),
    {
        let b: u8 = pre.spec_bits();
        match cap {
            Capability::ExceptionControl => {
                assert(((b | 1u8) as u8 & !1u8) as u8 == b) by (bit_vector)
                    requires (b & 1u8) == 0u8;
            },
            Capability::InterruptControl => {
                assert(((b | 2u8) as u8 & !2u8) as u8 == b) by (bit_vector)
                    requires (b & 2u8) == 0u8;
            },
            Capability::IoManagement => {
                assert(((b | 4u8) as u8 & !4u8) as u8 == b) by (bit_vector)
                    requires (b & 4u8) == 0u8;
            },
            Capability::MemoryManagement => {
                assert(((b | 8u8) as u8 & !8u8) as u8 == b) by (bit_vector)
                    requires (b & 8u8) == 0u8;
            },
            Capability::ProcessManagement => {
                assert(((b | 16u8) as u8 & !16u8) as u8 == b) by (bit_vector)
                    requires (b & 16u8) == 0u8;
            },
        }
    }

    /// Lemma: Roundtrip — setting after clearing restores the original bitfield
    /// when the bit was originally set.
    ///
    /// # Description
    ///
    /// If capability `cap` is set in `pre`, then `set(clear(pre, cap), cap)`
    /// returns the original bitfield. This proves that `clear` and `set` are
    /// inverses for the revoke/re-grant pattern.
    pub proof fn lemma_clear_set_roundtrip(pre: Capabilities, cap: Capability)
        requires
            pre.spec_has(cap),
        ensures
            ({
                let after_clear: Capabilities = Capabilities { bits: pre.spec_clear(cap) };
                after_clear.spec_set(cap) == pre.spec_bits()
            }),
    {
        let b: u8 = pre.spec_bits();
        match cap {
            Capability::ExceptionControl => {
                assert(((b & !1u8) as u8 | 1u8) as u8 == b) by (bit_vector)
                    requires (b & 1u8) != 0u8;
            },
            Capability::InterruptControl => {
                assert(((b & !2u8) as u8 | 2u8) as u8 == b) by (bit_vector)
                    requires (b & 2u8) != 0u8;
            },
            Capability::IoManagement => {
                assert(((b & !4u8) as u8 | 4u8) as u8 == b) by (bit_vector)
                    requires (b & 4u8) != 0u8;
            },
            Capability::MemoryManagement => {
                assert(((b & !8u8) as u8 | 8u8) as u8 == b) by (bit_vector)
                    requires (b & 8u8) != 0u8;
            },
            Capability::ProcessManagement => {
                assert(((b & !16u8) as u8 | 16u8) as u8 == b) by (bit_vector)
                    requires (b & 16u8) != 0u8;
            },
        }
    }

    /// Lemma: View equality implies bitfield equality.
    pub proof fn lemma_view_equality(a: &Capabilities, b: &Capabilities)
        requires
            a@ == b@,
        ensures
            a.spec_bits() == b.spec_bits(),
    {
    }

    /// Lemma: All capabilities are well-formed (when only valid bits are used).
    pub proof fn lemma_wf(&self)
        requires
            self.bits & 0b1110_0000u8 == 0u8,
        ensures
            self.wf(),
    {
    }

    /// Lemma: The Capability enum is closed — every instance is one of the 5
    /// known variants.
    ///
    /// # Description
    ///
    /// This proves the closed-world assumption: the `Capability` enum has exactly
    /// 5 variants, and any `Capability` value must be one of them. This is a
    /// consequence of Rust's exhaustive enum semantics and is verified here by
    /// exhaustive match. If a new variant were added to `Capability`, this match
    /// (and all others in this module) would fail to compile.
    pub proof fn lemma_enum_is_closed(cap: Capability)
        ensures
            cap == Capability::ExceptionControl
            || cap == Capability::InterruptControl
            || cap == Capability::IoManagement
            || cap == Capability::MemoryManagement
            || cap == Capability::ProcessManagement,
    {
        match cap {
            Capability::ExceptionControl => {},
            Capability::InterruptControl => {},
            Capability::IoManagement => {},
            Capability::MemoryManagement => {},
            Capability::ProcessManagement => {},
        }
    }

    /// Lemma: All capability masks only use bits 0..=4 (no upper bits set).
    ///
    /// # Description
    ///
    /// This connects the `spec_mask` values to the `wf()` invariant by proving
    /// that OR-ing any capability mask into a well-formed bitfield cannot set
    /// bits 5, 6, or 7. This is the foundation of `lemma_set_preserves_wf`.
    pub proof fn lemma_all_masks_valid(cap: Capability)
        ensures
            Self::spec_mask_is_valid(cap),
    {
        match cap {
            Capability::ExceptionControl => {
                assert(1u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            },
            Capability::InterruptControl => {
                assert(2u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            },
            Capability::IoManagement => {
                assert(4u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            },
            Capability::MemoryManagement => {
                assert(8u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            },
            Capability::ProcessManagement => {
                assert(16u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
            },
        }
    }
}

} // verus!
