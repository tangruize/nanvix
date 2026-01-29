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
