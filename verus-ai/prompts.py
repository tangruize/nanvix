# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Prompt templates for the Verus AI verification workflow.
"""

PROVER_PROMPT = """
Verify {source_path} using Verus.

Target: Create a verified Verus implementation in verus/{module_name}.rs

This is an OS kernel memory management component. Identify and prove the core
specifications, invariants, and safety/liveness properties that matter for correctness.

Requirements:
1. First run: cp {source_path} verus/{module_name}.rs
2. Define View types for abstract state representation
3. Define invariants that all operations must maintain
4. Add requires/ensures contracts for all functions (public and private impl fn)
5. Your proof must NOT contain any `assume` or `external_body` for core module functions
   (OK for low-level dependencies like raw memory operations)

Verification command: ./verus-ai/scripts/verify.sh {module_name}
  (This script runs verus, logs results, and commits changes automatically)

Dependencies already verified: {dependencies}

Iterate until verification passes. Document your work in the module comments.
""".strip()


REVIEWER_PROMPT = """
Review the Verus verification of {module_name}.

Original source: {source_path}
Verified code: verus/{module_name}.rs

This is an OS kernel memory management component. Evaluate if the verification
captures the essential correctness properties.

Review criteria:
1. COVERAGE: All functions in original source (public and private) have verified versions
2. SPECIFICATIONS: Specs capture intended behavior (not too weak, not too strong)
3. SOUNDNESS: No unjustified assume/external_body in core module
4. EQUIVALENCE: Verified code is semantically equivalent to original
5. INVARIANTS: State invariants are sufficient to prove correctness
6. PROPERTIES: Are the key safety and liveness properties identified and proven?

Verification command: ./verus-ai/scripts/verify.sh {module_name}

For each issue found, provide:
- Priority: Critical / High / Medium / Low
- Location: Function or spec name
- Description: What is wrong or missing
- Suggested Fix: How to address it

Output format (write to {review_file}):
```markdown
# Review: {module_name} ({model_name})

## Grade: [A+ / A / A- / B+ / B / B- / C / D / F]

## Issues Found

### Critical
- ...

### High
- ...

### Medium
- ...

### Low
- ...

## Positive Observations
- ...

## Summary
[Overall assessment and recommendations]
```
""".strip()


PROVER_FIX_PROMPT = """
A reviewer has identified issues in your Verus verification.

Review file: {review_file}
Module: verus/{module_name}.rs

Please address each issue:
1. If the issue is valid, fix it and explain your change
2. If the issue is not applicable, explain why it can be rejected

After fixing, run: cd verus && {verus_cmd}

Update the module until verification passes with all issues addressed.
""".strip()


PROVER_FIX_FRESH_PROMPT = """
You are starting a NEW verification session for {module_name}.

== CONTEXT ==
Original source: {source_path}
Verified code: verus/{module_name}.rs
Dependencies already verified: {dependencies}

This is an OS kernel memory management component. The existing verified code needs
improvements based on reviewer feedback.

== REVIEWER ISSUES ==
Review file(s): {review_file}

Please read the review file(s) carefully and address each issue:
1. Critical/High issues MUST be fixed
2. For Medium/Low issues, fix if valid or explain why rejection is justified
3. If the reviewer asks for missing functions, add them with proper verification

== REQUIREMENTS ==
1. Maintain or improve existing View types and invariants
2. All functions must have requires/ensures contracts
3. NO assume or unjustified external_body for core module functions
4. Verify semantic equivalence with original source

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}

Iterate until verification passes (0 errors) and all reviewer issues are addressed.
Document significant changes in module comments.
""".strip()


PROVER_RETRY_PROMPT = """
Verus verification failed. Please fix the errors and try again.

Module: verus/{module_name}.rs
Retry attempt: {retry_num} of {max_retries}

Verus output:
```
{verus_output}
```

Fix the verification errors and run: {verus_cmd}

Continue until verification passes (0 errors).
""".strip()


REVIEW_FOLLOWUP_PROMPT = """
The prover has addressed your previous review.

Previous review: {previous_review_file}
Updated module: verus/{module_name}.rs

Re-review with a critical and skeptical mindset:
1. Were previous issues ACTUALLY fixed, or just claimed to be fixed?
2. Verify the prover's claims - do not trust "not applicable" responses without evidence
3. Are there any new issues introduced by the fixes?
4. Is the verification now complete and sound?

Be fair but rigorous. If the prover rejects an issue, verify the rejection is justified.

Write your updated review to {review_file} using the same format.
Also write a simple result to {result_file}:
```
GRADE: [A+ / A / A- / B+ / B / B- / C / D / F]
PASSED: [YES / NO]
REMAINING_ISSUES: [count]
```
""".strip()


CHEATING_JUSTIFICATION_PROMPT = """
The verification of {module_name} contains potential cheating patterns that require justification.

Detected patterns:
{patterns_found}

For each pattern, explain:
1. Why it is necessary (or remove it if not necessary)
2. What guarantees are being assumed
3. How these assumptions can be validated

Write justifications as comments in the code or in a separate TRUST_BOUNDARY.md file.
""".strip()


#==================================================================================================
# Post-Processing Prompts: Simplification and Consistency Checking
#==================================================================================================

SIMPLIFY_PROOF_PROMPT = """
Simplify and clean up the Verus verification in verus/{module_name}.rs

== REFERENCE ==
Original source code: {source_path}
Verified code: verus/{module_name}.rs

Use the original source as reference to understand the intended behavior.
The verified code should remain semantically equivalent to the original.

== GOALS ==
1. **Remove redundant lemmas**: If a lemma is never called or its result is already implied by
   other postconditions, remove it.
2. **Remove redundant specs**: If a postcondition is logically implied by other postconditions,
   remove the redundant one. Example: if you have `ensures result >= 0` and `ensures result == x`
   where x is known non-negative, the first is redundant.
3. **Strengthen weak postconditions**: Especially for liveness properties. If a function can
   guarantee a stronger result, make the postcondition stronger. Example: change `ensures
   result.is_ok() ==> something` to also specify what happens on error.
4. **Condense verbose proofs**: Replace long inline proofs with calls to reusable lemmas.
   Move proof bodies into `proof fn lemma_xxx()` and call them from the exec function.
5. **Remove debug artifacts**: Remove `assert` statements that are not needed for verification,
   remove TODO/FIXME comments, remove commented-out code.

== STRICT CONSTRAINTS ==
- DO NOT change the semantics of any executable code
- DO NOT weaken any existing postconditions
- DO NOT remove postconditions that capture essential safety properties
- DO NOT add any `assume` statements
- DO NOT add any `admit` statements
- DO NOT add any `#[verifier::external_body]` annotations
- DO NOT add any `#[verifier::external]` annotations
- If you cannot simplify without adding these, leave the code as-is
- Verification MUST still pass after simplification

== PROCESS ==
1. First, list all lemmas and identify which are actually used
2. Analyze each spec function for redundancy
3. Check postconditions for weakness (especially liveness: can we guarantee more?)
4. Refactor verbose proofs into modular lemmas
5. Run verification: ./verus-ai/scripts/verify.sh {module_name}

== OUTPUT ==
After cleanup, provide a summary:
- Removed N redundant lemmas: [list names]
- Removed M redundant postconditions: [list them]
- Strengthened K postconditions: [list before/after]
- Condensed L proofs into lemmas: [list new lemma names]
- Lines reduced: before X -> after Y
- Cheating added: NONE (must be none!)

The goal is clean, auditable verification code that a human reviewer can read efficiently.
""".strip()


CHECK_CONSISTENCY_PROMPT = """
Check and FIX semantic consistency between the original source and verified Verus code.

Original source: {source_path}
Verified code: verus/{module_name}.rs

== YOUR TASK ==
1. Identify all inconsistencies between original and verified code
2. FIX inconsistencies that CAN be fixed (missing functions, wrong logic, etc.)
3. DOCUMENT inconsistencies that CANNOT be fixed (Verus limitations)
4. Run verification after fixes: ./verus-ai/scripts/verify.sh {module_name}

== CRITICAL CHECKS ==

1. **Function Coverage**: Every public AND private function in the original source must have
   a corresponding verified function.
   - **ACTION**: Add missing functions with proper verification
   - Do NOT skip complex functions

2. **Loop Transformations**: LLMs often transform `for` loops to `while` loops.
   - **ACTION**: Verify they are semantically equivalent (same iteration count, termination, side effects)
   - If not equivalent, FIX the loop logic

3. **Invented Functions**: Functions in Verus code that do NOT exist in original source.
   - spec fn / proof fn: OK (needed for verification)
   - New exec functions: REMOVE or justify why they preserve semantics

4. **Type Mismatches**: Check for silent type changes.
   - **FIXABLE**: Wrong integer types (u32 vs u64) → Fix to match original
   - **UNFIXABLE**: Pointer to usize (Verus limitation) → Document in report

5. **Control Flow Changes**: Early returns, error handling differences.
   - **ACTION**: Fix to match original control flow where possible

6. **Skipped Complexity**: Comments like "TODO: verify later" or "simplified for verification".
   - **ACTION**: Implement the skipped code properly

== UNFIXABLE ISSUES (Verus Limitations) ==
Some things cannot be fixed due to Verus/verification constraints. Document these:
- Raw pointers must be converted to usize (Verus doesn't support raw pointers well)
- Some unsafe code must use external_body
- Certain Rust features not supported in Verus

== CONSTRAINTS ==
- DO NOT add `assume`, `admit`, or unjustified `external_body`
- DO NOT change the semantic meaning of functions
- Verification MUST pass after your fixes

== OUTPUT ==

After making fixes, write a report to verus-ai-history/consistency/{module_name}.md:

```markdown
# Consistency Check: {module_name}

## Summary
- Issues Found: N
- Issues Fixed: M
- Unfixable Issues: K

## Fixed Issues
| Issue | Location | Fix Applied |
|-------|----------|-------------|
| Missing function bar() | line 42 | Added verified implementation |
| Wrong loop bounds | alloc() | Fixed iteration count |

## Unfixable Issues (Verus Limitations)
| Issue | Location | Reason Cannot Fix |
|-------|----------|-------------------|
| Pointer as usize | Frame.addr | Verus doesn't support raw pointers |

## Function Coverage
| Original Function | Verified Function | Status |
|-------------------|-------------------|--------|
| fn foo()          | fn foo()          | ✅ OK  |
| fn bar()          | fn bar()          | ✅ FIXED |

## Verification Status
- Before fixes: [PASS/FAIL]
- After fixes: [PASS/FAIL]

## Remaining Concerns
[Any issues that need human review]
```
""".strip()


STRENGTHEN_SPECS_PROMPT = """
Review and strengthen specifications in verus/{module_name}.rs

== REFERENCE ==
Original source code: {source_path}
Verified code: verus/{module_name}.rs

Use the original source as reference to understand the intended behavior.

== GOAL ==
Identify and fix WEAK postconditions that don't fully capture the function's guarantees.
A strong spec makes verification more useful by catching more bugs.

== COMMON WEAKNESS PATTERNS ==

1. **One-sided conditionals** (most common):
   - Weak: `ensures result.is_ok() ==> some_property`
   - Problem: Says nothing about the error case!
   - Strong: `ensures result.is_ok() <==> precondition_for_success`

2. **Trivially true specs**:
   - Weak: `ensures self.count >= 0` (always true for usize)
   - Strong: `ensures self.count == old(self).count + 1`

3. **Missing state change specs**:
   - Weak: `ensures result.is_ok()` (what changed?)
   - Strong: `ensures result.is_ok() ==> self@.contains(new_item)`

4. **Incomplete error specs**:
   - Weak: `ensures result.is_err() ==> true`
   - Strong: `ensures result.is_err() ==> specific_error_condition`

== CATEGORIES TO CHECK ==

### Liveness Properties (something good happens)
- Allocation: `has_capacity() ==> result.is_ok()` (not just "may succeed")
- Search: `contains(key) ==> result.is_some()` (not just "may find")
- Deallocation: `free_count == old(free_count) + 1` (resource returned)

### Safety Properties (nothing bad happens)
- Bounds: `index < len() ==> no_panic`
- Invariant preservation: `old(self).inv() ==> self.inv()`
- No aliasing: `result.addr != other.addr`

### Functional Correctness
- Getters: `result == self@.field` (exact value, not just "some value")
- Setters: `self@.field == new_value` (actually changed)
- Conversions: `result@ == self@` (no information loss)

== PROCESS ==

1. List all functions with their current postconditions
2. For each function, check:
   - Is success condition bidirectional (<==> not just ==>)?
   - Is error condition specified?
   - Are state changes fully described?
   - Can the spec be made more precise?
3. Strengthen weak specs and add proof if needed
4. Run verification: ./verus-ai/scripts/verify.sh {module_name}

== OUTPUT FORMAT ==

For each strengthened spec:
```
Function: alloc()
Before: ensures result.is_ok() ==> frame.inv()
Issue: One-sided conditional, missing error case
After: ensures
    old(self)@.has_capacity() ==> result.is_ok(),
    result.is_ok() ==> frame.inv(),
    result.is_err() ==> !old(self)@.has_capacity()
```

Summary:
- Strengthened N specs: [list function names]
- Added M error conditions
- Verification: PASS/FAIL
""".strip()


# Keep old name as alias for backwards compatibility.
STRENGTHEN_LIVENESS_PROMPT = STRENGTHEN_SPECS_PROMPT

