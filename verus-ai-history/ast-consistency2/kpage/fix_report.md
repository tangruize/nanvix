# Exec Consistency Fix: kpage

## Summary
- Mismatches fixed: 3 (documented)
- Missing functions added: 0
- Documented equivalences: 3

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) | [new_source.rs](new_source.rs) | [new_verus.rs](new_verus.rs) | Documented equivalence | `KernelPage { kframe }` ≡ `Self { kframe }` — purely syntactic; `Self` is an alias for `KernelPage` within the impl block. Proof block is ghost-only code. |
| `base` [base.diff](base.diff) | [base_source.rs](base_source.rs) | [base_verus.rs](base_verus.rs) | Documented Verus limitation | Original chain `base().into_page_address().into_virtual_address()` replaced by `base().into_raw_value()` because verus `FrameAddress` lacks `into_page_address()`/`into_virtual_address()` (simplified type). Under identity mapping both produce the same raw address value — see module-level equivalence proof in kpage.rs lines 42-58 and `proof_identity_mapping_justification` in kpage.proof.rs. |
| `frame_address` [frame_address.diff](frame_address.diff) | [frame_address_source.rs](frame_address_source.rs) | [frame_address_verus.rs](frame_address_verus.rs) | Documented equivalence | Exec body is identical (`self.kframe.base()`). AST mismatch is from proof ghost block and Verus return-type annotation `(result: FrameAddress)`. |
| `PageAddress` [struct_PageAddress_verus.rs](struct_PageAddress_verus.rs) (struct) | Kept — justified | Simplified replacement for `hal::mem::PageAddress` (wraps `usize` instead of `PageAligned<VirtualAddress>`). Required because the original type's dependency chain is not verified in Verus. |
| `PageAddress::new` | Kept — justified | Constructor for the simplified `PageAddress` [struct_PageAddress_verus.rs](struct_PageAddress_verus.rs) type. |
| `into_raw_value` [into_raw_value_verus.rs](into_raw_value_verus.rs) | Kept — justified | Accessor for `PageAddress` [struct_PageAddress_verus.rs](struct_PageAddress_verus.rs); mirrors the original `PageAddress::into_raw_value()` from `hal::mem`. |
| `get_pte_index` [get_pte_index_verus.rs](get_pte_index_verus.rs) | Kept — justified | Re-implementation of original `PageAddress::get_pte_index()` on the simplified type. |
| `page_address_eq` [page_address_eq_verus.rs](page_address_eq_verus.rs) | Kept — justified | Verified equality helper; provides non-`external_body` proof for the `PartialEq` implementation. |
| `eq` [eq_verus.rs](eq_verus.rs) (PartialEq) | Kept — justified | `external_body` PartialEq impl backed by verified `page_address_eq` [page_address_eq_verus.rs](page_address_eq_verus.rs) and `lemma_page_address_eq_correct`. |
| `pool_id` [pool_id_verus.rs](pool_id_verus.rs) | Kept — justified | Verification-only accessor exposing underlying frame's pool ID for provenance tracking (documented in module header). |

## Verification: PASS
```
verification results:: 16 verified, 0 errors
```
