#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Main workflow orchestrator for the Verus AI verification.

This script implements the Prover-Reviewer iteration workflow:
1. Prover generates initial Verus verification
2. Multiple reviewers (3 models) critique the verification
3. Prover addresses issues
4. Repeat until all reviewers assign grade A+
"""

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

from config import (
    CONSISTENCY_DIR,
    GRADE_THRESHOLD,
    HISTORY_DIR,
    INITIAL_PROVER_RETRIES,
    INTEGRITY_DIR,
    LOGS_DIR,
    MAX_INNER_ITERATIONS,
    ModuleConfig,
    PROJECT_ROOT,
    PROVER_MODEL,
    REVIEWER_MODELS,
    REVIEWS_DIR,
    VERUS_DIR,
    WORKFLOW_MODE,
    get_existing_modules,
    PROVER_TIMEOUT,
)
import config  # For dynamic MAX_OUTER_ITERATIONS access.
from copilot import CopilotSession, run_copilot, run_prover, run_reviewer, save_session
from guardrails import (
    CheatingReport,
    detect_cheating_in_module,
    git_commit_module,
    run_verus,
)
from prompts import (
    CHEATING_JUSTIFICATION_PROMPT,
    CHECK_CONSISTENCY_PROMPT,
    EXEC_CONSISTENCY_FIX_PROMPT,
    EXEC_CONSISTENCY_PROMPT,
    EXEC_CONSISTENCY_REVIEW_PROMPT,
    EXEC_INTEGRITY_FIX_PROMPT,
    EXEC_INTEGRITY_PROMPT,
    EXEC_INTEGRITY_REVIEW_PROMPT,
    IMPROVE_ABSTRACTION_FIX_PROMPT,
    IMPROVE_ABSTRACTION_PROMPT,
    IMPROVE_ABSTRACTION_REVIEW_PROMPT,
    PROVER_FIX_PROMPT,
    PROVER_FIX_FRESH_PROMPT,
    PROVER_PROMPT,
    PROVER_RETRY_PROMPT,
    REVIEW_FOLLOWUP_PROMPT,
    REVIEWER_PROMPT,
    SIMPLIFY_PROOF_PROMPT,
    SPEC_METHODOLOGY_FIX_PROMPT,
    SPEC_METHODOLOGY_PROMPT,
    SPEC_METHODOLOGY_REVIEW_PROMPT,
    STRENGTHEN_SPECS_PROMPT,
)


@dataclass
class ReviewResult:
    """Result of a single review."""

    model: str
    grade: str
    issues_count: int
    review_file: Path
    session_id: Optional[str] = None


@dataclass
class IterationResult:
    """Result of a prover-reviewer iteration."""

    iteration: int
    prover_success: bool
    verus_success: bool
    cheating_report: Optional[CheatingReport]
    reviews: List[ReviewResult]
    all_passed: bool


@dataclass
class WorkflowState:
    """Persistent state of the workflow."""

    module_name: str
    outer_iteration: int = 0  # Current outer round (all reviewers).
    inner_iteration: int = 0  # Current inner iteration (per reviewer).
    current_reviewer_idx: int = 0  # Which reviewer we're working with.
    prover_session_id: Optional[str] = None
    # Reviewer sessions are NOT persisted across outer iterations to avoid context pollution.
    completed: bool = False
    final_grades: Dict[str, str] = None  # Grade per reviewer.

    def __post_init__(self) -> None:
        if self.final_grades is None:
            self.final_grades = {}


def parse_grade(review_content: str) -> str:
    """Extract grade from review content."""
    # Look for patterns like "Grade: A+", "## Grade: B", etc.
    patterns = [
        r"##?\s*Grade\s*:\s*([A-F][+-]?)",
        r"Grade\s*:\s*([A-F][+-]?)",
        r"Overall\s+Grade\s*:\s*([A-F][+-]?)",
    ]

    for pattern in patterns:
        match = re.search(pattern, review_content, re.IGNORECASE)
        if match:
            return match.group(1).upper()

    return "?"


def grade_to_score(grade: str) -> int:
    """Convert letter grade to numeric score for comparison."""
    scores = {
        "A+": 13,
        "A": 12,
        "A-": 11,
        "B+": 10,
        "B": 9,
        "B-": 8,
        "C+": 7,
        "C": 6,
        "C-": 5,
        "D+": 4,
        "D": 3,
        "D-": 2,
        "F": 1,
        "?": 0,
    }
    return scores.get(grade, 0)


def is_passing_grade(grade: str) -> bool:
    """Check if grade meets threshold."""
    threshold_score = grade_to_score(GRADE_THRESHOLD)
    return grade_to_score(grade) >= threshold_score


def count_issues(review_content: str) -> int:
    """Count issues mentioned in review."""
    # Count bullet points and numbered items in issues sections.
    issue_count = 0
    in_issues_section = False

    for line in review_content.split("\n"):
        line_lower = line.lower()
        if "issue" in line_lower or "critical" in line_lower or "high" in line_lower:
            in_issues_section = True
        elif line.startswith("##"):
            in_issues_section = "issue" in line_lower

        if in_issues_section and (line.strip().startswith("-") or re.match(r"^\d+\.", line.strip())):
            issue_count += 1

    return issue_count


def save_workflow_state(state: WorkflowState) -> None:
    """Save workflow state to disk (in module subdirectory)."""
    module_log_dir = LOGS_DIR / state.module_name
    module_log_dir.mkdir(parents=True, exist_ok=True)
    state_file = module_log_dir / "workflow_state.json"

    with open(state_file, "w") as f:
        json.dump(asdict(state), f, indent=2)


def load_workflow_state(module_name: str) -> Optional[WorkflowState]:
    """Load workflow state from disk."""
    # Try new location first (module subdirectory).
    state_file = LOGS_DIR / module_name / "workflow_state.json"

    # Fall back to old location for backwards compatibility.
    if not state_file.exists():
        state_file = LOGS_DIR / f"workflow_state_{module_name}.json"

    if not state_file.exists():
        return None

    with open(state_file, "r") as f:
        data = json.load(f)

    # Handle migration from old format.
    if "current_iteration" in data:
        # Old format: migrate to new format.
        old_iteration = data.pop("current_iteration", 1)
        data["outer_iteration"] = old_iteration
        data["inner_iteration"] = 0
        data["current_reviewer_idx"] = 0
        # Convert old final_grade to final_grades dict.
        if "final_grade" in data:
            old_grade = data.pop("final_grade")
            if old_grade:
                data["final_grades"] = {"claude": old_grade, "gpt": old_grade, "gemini": old_grade}
        # Remove old reviewer_sessions (we don't reuse sessions anymore).
        data.pop("reviewer_sessions", None)

    return WorkflowState(**data)


def _module_fmt(module: ModuleConfig) -> dict:
    """Return common format kwargs for prompt templates from a ModuleConfig."""
    return {
        "module_name": module.name,
        "source_path": module.source_path,
        "output_dir": module.output_dir(),
        "file_stem": module.file_stem,
        "type_name": module.file_stem.title().replace("_", ""),
        "verus_cmd": module.verus_cmd(),
    }


def run_initial_prover(module: ModuleConfig) -> tuple[bool, CopilotSession]:
    """
    Run the initial prover to generate Verus code with retry mechanism.

    The prover will retry up to INITIAL_PROVER_RETRIES times if Verus verification fails.
    Even if all retries fail, we continue to the review phase (reviewers may help identify issues).

    Returns:
        Tuple of (verus_passed, session).
    """
    # Get existing verified modules as context for dependencies.
    existing_modules = get_existing_modules()
    deps_str = ", ".join(existing_modules) if existing_modules else "none"

    fmt = _module_fmt(module)
    prompt = PROVER_PROMPT.format(
        **fmt,
        dependencies=deps_str,
    )

    # Commit: prover start.
    git_commit_module(module.name, f"[verus-ai] Prover START: {module.name}")

    print(f"[PROVER] Generating Verus verification for {module.name}...")
    output, session = run_prover(prompt, PROVER_MODEL, module_name=module.name)

    # Check if Verus passes.
    verus_passed, verus_output = run_verus(module.name)
    print(f"[VERUS] Initial check: {'PASSED' if verus_passed else 'FAILED'}")

    # Retry loop if Verus fails.
    retry_num = 0
    while not verus_passed and retry_num < INITIAL_PROVER_RETRIES:
        retry_num += 1
        print(f"\n[PROVER] Retry {retry_num}/{INITIAL_PROVER_RETRIES}...")

        # Commit: retry start.
        git_commit_module(module.name, f"[verus-ai] Prover RETRY {retry_num}: {module.name}")

        retry_prompt = PROVER_RETRY_PROMPT.format(
            **fmt,
            retry_num=retry_num,
            max_retries=INITIAL_PROVER_RETRIES,
            verus_output=verus_output[:2000],  # Truncate to avoid token overflow.
        )

        # Use run_copilot to resume the session.
        output, session = run_copilot(
            retry_prompt,
            PROVER_MODEL,
            session=session,
            timeout=PROVER_TIMEOUT,
            log_prefix=f"prover_retry{retry_num}",
            module_name=module.name,
        )

        # Check Verus again.
        verus_passed, verus_output = run_verus(module.name)
        print(f"[VERUS] Retry {retry_num}: {'PASSED' if verus_passed else 'FAILED'}")

    # Commit: prover end.
    status = "PASSED" if verus_passed else f"FAILED after {retry_num} retries"
    git_commit_module(module.name, f"[verus-ai] Prover END: {module.name} ({status}, session: {session.session_id or 'unknown'})")

    if not verus_passed:
        print(f"[PROVER] WARNING: Verus still failing after {retry_num} retries, continuing to review phase...")

    return verus_passed, session


def run_single_review(
    module: ModuleConfig,
    model: str,
    outer_round: int,
    inner_attempt: int,
    session: Optional[CopilotSession] = None,
) -> ReviewResult:
    """
    Run a single reviewer.

    Parameters:
        module: Module configuration.
        model: Reviewer model name.
        outer_round: Current outer round (1-based).
        inner_attempt: Current inner attempt within this reviewer (1-based).
        session: Optional session to resume (within same reviewer's inner loop).
    """
    model_short = model.split("-")[0]  # e.g., "claude" from "claude-opus-4.6"
    # Organize reviews by module subdirectory.
    # Naming: {model}_r{outer}_a{inner}.md (e.g., claude_r1_a2.md)
    module_review_dir = REVIEWS_DIR / module.name
    module_review_dir.mkdir(parents=True, exist_ok=True)
    review_file = module_review_dir / f"{model_short}_r{outer_round}_a{inner_attempt}.md"
    result_file = module_review_dir / f"{model_short}_r{outer_round}_a{inner_attempt}_result.txt"

    if session and session.session_id and inner_attempt > 1:
        # Follow-up review with result file (within same reviewer's inner loop).
        previous_review = module_review_dir / f"{model_short}_r{outer_round}_a{inner_attempt-1}.md"
        prompt = REVIEW_FOLLOWUP_PROMPT.format(
            previous_review_file=previous_review,
            module_name=module.name,
            output_dir=module.output_dir(),
            file_stem=module.file_stem,
            review_file=review_file,
            result_file=result_file,
        )
    else:
        # Initial review (new reviewer or new outer round).
        prompt = REVIEWER_PROMPT.format(
            module_name=module.name,
            source_path=module.source_path,
            output_dir=module.output_dir(),
            file_stem=module.file_stem,
            verus_cmd=module.verus_cmd(),
            review_file=review_file,
            model_name=model,
        )

    # Commit: reviewer start.
    git_commit_module(module.name, f"[verus-ai] Reviewer START: {module.name} ({model_short} r{outer_round}a{inner_attempt})")

    print(f"[REVIEWER] Running {model} (round {outer_round}, attempt {inner_attempt})...")
    output, new_session = run_reviewer(prompt, model, session, module_name=module.name)

    # Commit: reviewer end.
    git_commit_module(module.name, f"[verus-ai] Reviewer END: {module.name} ({model_short} r{outer_round}a{inner_attempt})")

    # Read the review file if it exists.
    grade = "?"
    issues_count = 0

    # First try to read from result file (simpler format).
    if result_file.exists():
        result_content = result_file.read_text()
        grade = parse_grade(result_content)
        # Parse REMAINING_ISSUES from result file.
        issues_match = re.search(r"REMAINING_ISSUES\s*:\s*(\d+)", result_content)
        if issues_match:
            issues_count = int(issues_match.group(1))

    # Fall back to review file.
    if grade == "?" and review_file.exists():
        content = review_file.read_text()
        grade = parse_grade(content)
        issues_count = count_issues(content)

    return ReviewResult(
        model=model,
        grade=grade,
        issues_count=issues_count,
        review_file=review_file,
        session_id=new_session.session_id,
    )


def run_prover_fix(
    module: ModuleConfig,
    review_files: List[Path],
    session: CopilotSession,
    fresh_context: bool = False,
) -> tuple[bool, CopilotSession]:
    """Run prover to fix issues from reviews.
    
    Parameters:
        module: Module configuration.
        review_files: List of review files to address.
        session: Copilot session (may be None for fresh start).
        fresh_context: If True, use full context prompt for new session.
    """
    # Combine review file references.
    review_refs = "\n".join([f"- {rf}" for rf in review_files])

    if fresh_context:
        # Use full context prompt for fresh prover session.
        existing_modules = get_existing_modules()
        deps_str = ", ".join(existing_modules) if existing_modules else "none"
        prompt = PROVER_FIX_FRESH_PROMPT.format(
            review_file=review_refs,
            module_name=module.name,
            source_path=module.source_path,
            output_dir=module.output_dir(),
            file_stem=module.file_stem,
            dependencies=deps_str,
        )
    else:
        prompt = PROVER_FIX_PROMPT.format(
            review_file=review_refs,
            module_name=module.name,
            output_dir=module.output_dir(),
            file_stem=module.file_stem,
            verus_cmd=module.verus_cmd(),
        )

    # Commit: prover fix start.
    session_info = 'fresh' if fresh_context else (session.session_id or 'new')
    git_commit_module(module.name, f"[verus-ai] Prover FIX START: {module.name} (session: {session_info})")

    print(f"[PROVER] Fixing issues from {len(review_files)} reviews...")
    print(f"[PROVER] Session mode: {'fresh context' if fresh_context else 'resuming ' + (session.session_id or 'new')}")

    # Use run_copilot directly to pass the session for resume.
    # For fresh_context, don't pass session to start a new one.
    output, new_session = run_copilot(
        prompt,
        PROVER_MODEL,
        session=None if fresh_context else session,
        timeout=PROVER_TIMEOUT,
        log_prefix="prover_fix_fresh" if fresh_context else "prover_fix",
        module_name=module.name,
    )

    # Commit: prover fix end.
    git_commit_module(module.name, f"[verus-ai] Prover FIX END: {module.name}")

    return True, new_session


def run_one_on_one_review(
    module: ModuleConfig,
    model: str,
    outer_round: int,
    state: WorkflowState,
    fresh_prover: bool = False,
) -> tuple[str, bool]:
    """
    Run one-on-one review with a single reviewer until A+ or max attempts.

    The reviewer and prover engage in a back-and-forth loop:
    - Reviewer reviews the code
    - If not A+, prover fixes issues
    - Reviewer re-reviews
    - Repeat until A+ or max inner iterations

    Each reviewer gets a FRESH session (no session from previous reviewer or round).

    Parameters:
        module: Module configuration.
        model: Reviewer model name.
        outer_round: Current outer round (1-based).
        state: Workflow state.
        fresh_prover: If True, use fresh context for prover fixes.

    Returns:
        Tuple of (final_grade, passed).
    """
    model_short = model.split("-")[0]
    reviewer_session: Optional[CopilotSession] = None  # Fresh session for each reviewer.

    print(f"\n{'='*60}")
    print(f"ONE-ON-ONE: {model} (Round {outer_round})")
    print(f"{'='*60}")

    for inner_attempt in range(1, MAX_INNER_ITERATIONS + 1):
        print(f"\n[ATTEMPT {inner_attempt}/{MAX_INNER_ITERATIONS}]")

        # Step 1: Run Verus verification.
        print("[VERUS] Running verification...")
        verus_success, verus_output = run_verus(module.name)
        print(f"[VERUS] {'PASSED' if verus_success else 'FAILED'}")

        if not verus_success:
            print(f"[VERUS] Output: {verus_output[:500]}")

        # Step 2: Check for cheating.
        cheating_report = detect_cheating_in_module(module.name)
        if cheating_report.has_cheating():
            print(f"[GUARDRAILS] WARNING: {cheating_report.summary()}")

        # Step 3: Run reviewer.
        review = run_single_review(
            module, model, outer_round, inner_attempt, reviewer_session
        )
        reviewer_session = CopilotSession(session_id=review.session_id, model=model)

        print(f"[REVIEWER] {model_short}: Grade={review.grade}, Issues={review.issues_count}")

        # Step 4: Check if passed.
        if is_passing_grade(review.grade) and verus_success:
            print(f"[SUCCESS] {model_short} passed with grade {review.grade}")
            return review.grade, True

        # Step 5: If not passed and not last attempt, run prover fix.
        if inner_attempt < MAX_INNER_ITERATIONS:
            if review.review_file.exists():
                print(f"[PROVER] Fixing issues from {model_short}...")
                # Determine if we need fresh context:
                # - fresh_prover=True AND no current session = use fresh context
                # - Otherwise, resume existing session
                use_fresh_context = fresh_prover and state.prover_session_id is None
                prover_session = CopilotSession(
                    session_id=state.prover_session_id, model=PROVER_MODEL
                )
                _, new_prover_session = run_prover_fix(
                    module, [review.review_file], prover_session, fresh_context=use_fresh_context
                )
                state.prover_session_id = new_prover_session.session_id
                save_workflow_state(state)
            else:
                print(f"[WARNING] No review file found for {model_short}")
        else:
            print(f"[TIMEOUT] Max inner attempts reached for {model_short}")

    # Return last grade even if not passed.
    return review.grade, False


def run_workflow(
    source_path: str,
    module_name: Optional[str] = None,
    resume: bool = False,
    fresh_prover: bool = False,
    output_subdir: Optional[str] = None,
    file_stem: Optional[str] = None,
) -> bool:
    """
    Run the complete verification workflow for a module.

    New double-loop structure:
    - Outer loop: All reviewers get a chance (max MAX_OUTER_ITERATIONS rounds)
    - Inner loop: Each reviewer has one-on-one with prover (max MAX_INNER_ITERATIONS attempts)

    Reviewers do NOT share sessions across outer rounds (to avoid context pollution).

    Parameters:
        source_path: Path to the source file to verify.
        module_name: Optional module name.
        resume: Whether to resume from saved state.
        fresh_prover: If True, start new prover session each outer round (default: False).
        output_subdir: Optional output subdirectory under verus/split/.
        file_stem: Optional file stem for the three-file split.

    Returns:
        True if verification succeeded (all reviewers passed).
    """
    # Create module config.
    module = ModuleConfig.from_source_path(
        source_path,
        module_name,
        output_subdir=output_subdir,
        file_stem=file_stem,
    )

    # Check source file exists.
    full_source_path = PROJECT_ROOT / module.source_path
    if not full_source_path.exists():
        print(f"ERROR: Source file not found: {full_source_path}")
        return False

    # Load or create state.
    state = load_workflow_state(module.name) if resume else None
    if state is None:
        state = WorkflowState(module_name=module.name)

    print(f"\n{'#'*60}")
    print(f"VERUS AI VERIFICATION WORKFLOW")
    print(f"Module: {module.name}")
    print(f"Source: {module.source_path}")
    print(f"Output: {module.output_dir()}/")
    print(f"Files:  {module.file_stem}.rs, {module.file_stem}.spec.rs, {module.file_stem}.proof.rs")
    print(f"Max outer rounds: {config.MAX_OUTER_ITERATIONS}")
    print(f"Max inner attempts per reviewer: {MAX_INNER_ITERATIONS}")
    if resume:
        print(f"Resuming from outer round: {state.outer_iteration}")
    print(f"{'#'*60}")

    # Step 0: Run initial prover if starting fresh.
    if state.outer_iteration == 0 and not resume:
        success, prover_session = run_initial_prover(module)
        state.prover_session_id = prover_session.session_id
        state.outer_iteration = 1
        save_workflow_state(state)

        # Git commit.
        git_commit_module(module.name, f"[verus-ai] Initial verification of {module.name}")

    # Outer loop: rounds where all reviewers participate.
    start_round = state.outer_iteration if resume else 1

    # If fresh_prover and resuming, clear the session at the very start.
    if fresh_prover and resume and state.prover_session_id is not None:
        print(f"\n[FRESH PROVER] Clearing previous prover session for fresh start")
        state.prover_session_id = None
        save_workflow_state(state)

    for outer_round in range(start_round, config.MAX_OUTER_ITERATIONS + 1):
        state.outer_iteration = outer_round

        # Optionally reset prover session at the start of each outer round (except first).
        if fresh_prover and outer_round > start_round:
            print(f"\n[FRESH PROVER] Starting new prover session for round {outer_round}")
            state.prover_session_id = None

        save_workflow_state(state)

        print(f"\n{'#'*60}")
        print(f"OUTER ROUND {outer_round}/{config.MAX_OUTER_ITERATIONS}")
        if fresh_prover:
            print(f"Prover session: {'new' if state.prover_session_id is None else 'continuing'}")
        print(f"{'#'*60}")

        round_grades: Dict[str, str] = {}
        all_passed = True

        # Inner loop: one-on-one with each reviewer.
        for reviewer_idx, model in enumerate(REVIEWER_MODELS):
            model_short = model.split("-")[0]

            grade, passed = run_one_on_one_review(
                module, model, outer_round, state, fresh_prover=fresh_prover
            )
            round_grades[model_short] = grade
            state.final_grades[model_short] = grade
            save_workflow_state(state)

            if not passed:
                all_passed = False

        # Check if all passed.
        if all_passed:
            print(f"\n{'='*60}")
            print(f"SUCCESS: All reviewers passed at round {outer_round}")
            grades_str = ", ".join([f"{k}: {v}" for k, v in round_grades.items()])
            print(f"Grades: {grades_str}")
            print(f"{'='*60}")

            # Final commit.
            git_commit_module(
                module.name,
                f"[verus-ai] Verified {module.name} ({grades_str})"
            )

            state.completed = True
            save_workflow_state(state)
            return True

        # Not all passed, continue to next outer round.
        grades_str = ", ".join([f"{k}: {v}" for k, v in round_grades.items()])
        print(f"\n[ROUND {outer_round}] Not all passed. Grades: {grades_str}")
        git_commit_module(
            module.name,
            f"[verus-ai] Round {outer_round} complete for {module.name} ({grades_str})"
        )

    print(f"\n{'='*60}")
    print(f"TIMEOUT: Max outer rounds ({config.MAX_OUTER_ITERATIONS}) reached")
    grades_str = ", ".join([f"{k}: {v}" for k, v in state.final_grades.items()])
    print(f"Final grades: {grades_str}")
    print(f"{'='*60}")
    return False


#==================================================================================================
# Post-Processing Commands: Simplify, Consistency Check, Strengthen
#==================================================================================================

def find_source_path(module_name: str) -> Optional[str]:
    """
    Try to find the original source file for a module.

    Searches common patterns in the Nanvix codebase.
    Returns the relative path if found, None otherwise.
    """
    # Common patterns for source files in Nanvix.
    possible_paths = [
        # Libraries (most common for verified modules).
        f"src/libs/{module_name}/src/lib.rs",
        f"src/libs/{module_name}/src/{module_name}.rs",
        # Physical memory management.
        f"src/kernel/src/mm/phys/{module_name}.rs",
        f"src/kernel/src/mm/phys/{module_name}/mod.rs",
        # Virtual memory management.
        f"src/kernel/src/mm/virt/{module_name}.rs",
        f"src/kernel/src/mm/virt/{module_name}/mod.rs",
        # General mm.
        f"src/kernel/src/mm/{module_name}.rs",
        f"src/kernel/src/mm/{module_name}/mod.rs",
        # Process management (scheduler).
        f"src/kernel/src/pm/{module_name}.rs",
        f"src/kernel/src/pm/{module_name}/mod.rs",
        # PM thread states.
        f"src/kernel/src/pm/thread/{module_name}.rs",
        # PM process states.
        f"src/kernel/src/pm/process/{module_name}.rs",
        f"src/kernel/src/pm/process/state/{module_name}.rs",
        f"src/kernel/src/pm/process/manager/{module_name}.rs",
        f"src/kernel/src/pm/process/manager/mod.rs",
        # PM synchronization.
        f"src/kernel/src/pm/sync/{module_name}.rs",
        # Kernel libs.
        f"src/kernel/src/libs/{module_name}.rs",
        f"src/kernel/src/libs/{module_name}/mod.rs",
        # HAL/arch.
        f"src/kernel/src/hal/{module_name}.rs",
        f"src/kernel/src/hal/arch/x86/mem/{module_name}.rs",
        # System libs PM types.
        f"src/libs/sys/src/sys/pm/{module_name}.rs",
    ]

    for path in possible_paths:
        if (PROJECT_ROOT / path).exists():
            return path

    return None


def run_simplify(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Run proof simplification on a verified module.

    Removes redundant lemmas/specs, condenses verbose proofs, removes debug artifacts.
    Requires original source as reference and prohibits adding assume/external_body.
    """
    verus_file = VERUS_DIR / f"{module_name}.rs"
    if not verus_file.exists():
        print(f"ERROR: Verus file not found: {verus_file}")
        return False

    # Find source path if not provided.
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"WARNING: Could not auto-detect source for {module_name}")
            print("Tip: Use --source <path> to specify the original source file")
            source_path = f"(unknown - please check verus/{module_name}.rs comments for reference)"

    print(f"\n{'#'*60}")
    print(f"SIMPLIFY: {module_name}")
    print(f"Original: {source_path}")
    print(f"Verified: verus/{module_name}.rs")
    print(f"{'#'*60}")

    # Generate timestamp for report filename.
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    prompt = SIMPLIFY_PROOF_PROMPT.format(
        module_name=module_name,
        source_path=source_path,
        output_dir=f"verus/split/{module_name}",
        file_stem=module_name,
        timestamp=timestamp,
    )

    # Commit before simplification.
    git_commit_module(module_name, f"[verus-ai] Simplify START: {module_name}")

    output, session = run_copilot(
        prompt,
        PROVER_MODEL,
        timeout=PROVER_TIMEOUT,
        log_prefix="simplify",
        module_name=module_name,
    )

    # Commit after simplification.
    git_commit_module(module_name, f"[verus-ai] Simplify END: {module_name}")

    # Check for cheating patterns introduced.
    print("\n[GUARDRAILS] Checking for new assume/external_body...")
    cheating_report = detect_cheating_in_module(module_name)
    if cheating_report.has_cheating():
        print(f"[GUARDRAILS] ⚠️ WARNING: Cheating patterns detected!")
        print(cheating_report.summary())

    # Run verification to ensure it still passes.
    print("\n[VERUS] Verifying after simplification...")
    verus_success, verus_output = run_verus(module_name)
    print(f"[VERUS] {'PASSED' if verus_success else 'FAILED'}")

    if not verus_success:
        print(f"[WARNING] Verification failed after simplification!")
        print(f"[VERUS] Output: {verus_output[:1000]}")

    return verus_success


def run_consistency_check(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Check and fix semantic consistency between original source and verified code.

    Identifies missing functions, loop transformations, invented functions, etc.
    Attempts to fix issues where possible, documents unfixable issues (Verus limitations).
    """
    verus_file = VERUS_DIR / f"{module_name}.rs"
    if not verus_file.exists():
        print(f"ERROR: Verus file not found: {verus_file}")
        return False

    # Try to find source path if not provided.
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"ERROR: Could not find original source for {module_name}")
            print("Please specify with --source <path>")
            return False

    print(f"\n{'#'*60}")
    print(f"CONSISTENCY CHECK & FIX: {module_name}")
    print(f"Original: {source_path}")
    print(f"Verified: verus/{module_name}.rs")
    print(f"{'#'*60}")

    # Generate timestamp for report filename.
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    # Commit before consistency check/fix.
    git_commit_module(module_name, f"[verus-ai] Consistency check START: {module_name}")

    prompt = CHECK_CONSISTENCY_PROMPT.format(
        module_name=module_name,
        source_path=source_path,
        output_dir=f"verus/split/{module_name}",
        file_stem=module_name,
        timestamp=timestamp,
    )

    output, session = run_copilot(
        prompt,
        PROVER_MODEL,
        timeout=PROVER_TIMEOUT,
        log_prefix="consistency",
        module_name=module_name,
    )

    # Commit after consistency check/fix.
    git_commit_module(module_name, f"[verus-ai] Consistency check END: {module_name}")

    # Check for cheating patterns introduced.
    print("\n[GUARDRAILS] Checking for assume/external_body...")
    cheating_report = detect_cheating_in_module(module_name)
    if cheating_report.has_cheating():
        print(f"[GUARDRAILS] ⚠️ WARNING: Cheating patterns detected!")
        print(cheating_report.summary())

    # Run verification to ensure it still passes.
    print("\n[VERUS] Verifying after consistency fixes...")
    verus_success, verus_output = run_verus(module_name)
    print(f"[VERUS] {'PASSED' if verus_success else 'FAILED'}")

    if not verus_success:
        print(f"[WARNING] Verification failed after consistency fixes!")
        print(f"[VERUS] Output: {verus_output[:1000]}")

    # Check if report was created.
    CONSISTENCY_DIR.mkdir(parents=True, exist_ok=True)
    report_file = CONSISTENCY_DIR / f"{module_name}.md"
    if report_file.exists():
        print(f"\n[REPORT] Consistency report saved to: {report_file}")
        content = report_file.read_text()
        # Count issues.
        if "Unfixable Issues" in content:
            print("[INFO] Some issues could not be fixed due to Verus limitations - see report")
    else:
        print(f"[WARNING] No report file created at {report_file}")

    return verus_success


def run_strengthen_liveness(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Review and strengthen specifications in a verified module.

    Fixes weak postconditions: one-sided conditionals, missing error specs,
    incomplete state change specs, etc. Includes liveness, safety, and functional correctness.
    """
    verus_file = VERUS_DIR / f"{module_name}.rs"
    if not verus_file.exists():
        print(f"ERROR: Verus file not found: {verus_file}")
        return False

    # Find source path if not provided.
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"WARNING: Could not find source path for {module_name}")
            source_path = "(source not found - use original implementation as reference)"

    print(f"\n{'#'*60}")
    print(f"STRENGTHEN SPECS: {module_name}")
    print(f"Source: {source_path}")
    print(f"{'#'*60}")

    # Generate timestamp for report filename.
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

    prompt = STRENGTHEN_SPECS_PROMPT.format(
        module_name=module_name,
        source_path=source_path,
        output_dir=f"verus/split/{module_name}",
        file_stem=module_name,
        timestamp=timestamp,
    )

    # Commit before strengthening.
    git_commit_module(module_name, f"[verus-ai] Strengthen specs START: {module_name}")

    output, session = run_copilot(
        prompt,
        PROVER_MODEL,
        timeout=PROVER_TIMEOUT,
        log_prefix="strengthen",
        module_name=module_name,
    )

    # Commit after strengthening.
    git_commit_module(module_name, f"[verus-ai] Strengthen specs END: {module_name}")

    # Run verification to ensure it still passes.
    print("\n[VERUS] Verifying after strengthening...")
    verus_success, verus_output = run_verus(module_name)
    print(f"[VERUS] {'PASSED' if verus_success else 'FAILED'}")

    if not verus_success:
        print(f"[WARNING] Verification failed after strengthening!")
        print(f"[VERUS] Output: {verus_output[:1000]}")

    return verus_success


def _run_single_step_with_review(
    module_name: str,
    source_path: str,
    step_name: str,
    prover_prompt: str,
    review_prompt_template: str,
    fix_prompt_template: str,
    review_model: str = "claude-opus-4.6",
    report_file: Optional[str] = None,
) -> bool:
    """
    Run a single improvement step with one round of review + fix.

    Flow: prover runs → reviewer reviews → prover fixes → verify.

    Parameters:
        module_name: Module name.
        source_path: Path to original source.
        step_name: Step name for logs (e.g., "improve-abstraction").
        prover_prompt: The initial prover prompt.
        review_prompt_template: Template for review (needs review_file, model_name).
        fix_prompt_template: Template for fix (needs review_file).
        review_model: Model for the reviewer.
        report_file: Optional report file path (for exec-integrity).

    Returns:
        True if verification passes at the end.
    """
    # Derive module config for output_dir/file_stem.
    module = _find_module_config(module_name, source_path)
    fmt = _module_fmt(module)

    # Ensure review directory exists.
    module_review_dir = REVIEWS_DIR / module_name
    module_review_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    review_file = module_review_dir / f"{step_name}_{timestamp}.md"

    # Step 1: Prover runs.
    git_commit_module(module_name, f"[verus-ai] {step_name} START: {module_name}")
    print(f"[PROVER] Running {step_name} for {module_name}...")
    output, prover_session = run_copilot(
        prover_prompt,
        PROVER_MODEL,
        timeout=PROVER_TIMEOUT,
        log_prefix=f"{step_name}_prover",
        module_name=module_name,
    )
    git_commit_module(module_name, f"[verus-ai] {step_name} prover END: {module_name}")

    # Step 2: Verify after prover.
    verus_success, verus_output = run_verus(module_name)
    print(f"[VERUS] After prover: {'PASSED' if verus_success else 'FAILED'}")

    # Step 3: Reviewer reviews.
    review_fmt = {
        **fmt,
        "review_file": str(review_file),
        "model_name": review_model,
    }
    if report_file:
        review_fmt["report_file"] = report_file
    review_prompt = review_prompt_template.format(**review_fmt)

    print(f"[REVIEWER] Running {review_model} review for {step_name}...")
    review_output, reviewer_session = run_reviewer(
        review_prompt, review_model, module_name=module_name
    )
    git_commit_module(module_name, f"[verus-ai] {step_name} review END: {module_name}")

    # Parse grade from review.
    grade = "?"
    if review_file.exists():
        grade = parse_grade(review_file.read_text())
    print(f"[REVIEWER] Grade: {grade}")

    # Step 4: Prover fixes based on review (if review file exists).
    if review_file.exists() and not is_passing_grade(grade):
        fix_fmt = {
            **fmt,
            "review_file": str(review_file),
        }
        if report_file:
            fix_fmt["report_file"] = report_file
        fix_prompt = fix_prompt_template.format(**fix_fmt)

        print(f"[PROVER] Fixing issues from review...")
        fix_output, fix_session = run_copilot(
            fix_prompt,
            PROVER_MODEL,
            session=prover_session,
            timeout=PROVER_TIMEOUT,
            log_prefix=f"{step_name}_fix",
            module_name=module_name,
        )
        git_commit_module(module_name, f"[verus-ai] {step_name} fix END: {module_name}")

    # Step 5: Final verification.
    verus_success, verus_output = run_verus(module_name)
    print(f"[VERUS] Final: {'PASSED' if verus_success else 'FAILED'}")

    if not verus_success:
        print(f"[WARNING] Verification failed after {step_name}!")
        print(f"[VERUS] Output: {verus_output[:1000]}")

    return verus_success


def _find_module_config(module_name: str, source_path: str) -> ModuleConfig:
    """Find or create a ModuleConfig for a module name."""
    # Infer output_subdir from source path.
    # e.g., src/kernel/src/pm/process/state/runnable.rs -> kernel/pm/process/state
    path = Path(source_path)
    parts = path.parts
    # Find "kernel" in parts and build from there.
    output_subdir = module_name
    file_stem = module_name
    try:
        kernel_idx = list(parts).index("kernel")
        # Skip "kernel/src" -> take from parts[kernel_idx], skip src.
        relevant = [p for p in parts[kernel_idx:] if p != "src"]
        # Remove filename to get directory.
        output_subdir = "/".join(relevant[:-1])
        file_stem = path.stem
    except ValueError:
        # Try libs path.
        try:
            libs_idx = list(parts).index("libs")
            relevant = [p for p in parts[libs_idx:] if p != "src"]
            output_subdir = "/".join(relevant[:-1])
            file_stem = path.stem
        except ValueError:
            pass

    return ModuleConfig(
        name=module_name,
        source_path=path,
        verify_module=module_name,
        output_subdir=output_subdir,
        file_stem=file_stem,
    )


def run_improve_abstraction(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Improve spec abstraction for a module with review.

    Adds View-level abstract state transition functions to .spec.rs files.
    Followed by one round of review + fix.
    """
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"ERROR: Could not find source for {module_name}")
            return False

    module = _find_module_config(module_name, source_path)
    fmt = _module_fmt(module)

    print(f"\n{'#'*60}")
    print(f"IMPROVE ABSTRACTION: {module_name}")
    print(f"Source: {source_path}")
    print(f"Output: {module.output_dir()}/")
    print(f"{'#'*60}")

    prompt = IMPROVE_ABSTRACTION_PROMPT.format(**fmt)

    return _run_single_step_with_review(
        module_name=module_name,
        source_path=source_path,
        step_name="improve-abstraction",
        prover_prompt=prompt,
        review_prompt_template=IMPROVE_ABSTRACTION_REVIEW_PROMPT,
        fix_prompt_template=IMPROVE_ABSTRACTION_FIX_PROMPT,
    )


def run_exec_integrity(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Check exec code integrity for a module with review.

    Compares verified exec code against original source, fixes logic changes,
    and produces integrity report. Followed by one round of review + fix.
    """
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"ERROR: Could not find source for {module_name}")
            return False

    module = _find_module_config(module_name, source_path)
    fmt = _module_fmt(module)

    print(f"\n{'#'*60}")
    print(f"EXEC INTEGRITY: {module_name}")
    print(f"Source: {source_path}")
    print(f"Output: {module.output_dir()}/")
    print(f"{'#'*60}")

    # Create integrity report directory.
    INTEGRITY_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_file = str(INTEGRITY_DIR / f"{module_name}_{timestamp}.md")

    prompt = EXEC_INTEGRITY_PROMPT.format(**fmt, report_file=report_file)

    return _run_single_step_with_review(
        module_name=module_name,
        source_path=source_path,
        step_name="exec-integrity",
        prover_prompt=prompt,
        review_prompt_template=EXEC_INTEGRITY_REVIEW_PROMPT,
        fix_prompt_template=EXEC_INTEGRITY_FIX_PROMPT,
        report_file=report_file,
    )


def run_spec_methodology(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Check and fix spec methodology compliance using tree-sitter analysis.

    Runs check_spec_methodology.py to detect violations, then has AI fix them.
    Followed by one round of review + fix.
    """
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"ERROR: Could not find source for {module_name}")
            return False

    module = _find_module_config(module_name, source_path)
    fmt = _module_fmt(module)

    print(f"\n{'#'*60}")
    print(f"SPEC METHODOLOGY: {module_name}")
    print(f"Source: {source_path}")
    print(f"Output: {module.output_dir()}/")
    print(f"{'#'*60}")

    # Step 0: Run tree-sitter analysis to generate violations report.
    import subprocess
    verus_dir = str(PROJECT_ROOT / module.output_dir().replace("verus/split/", ""))
    # The output_dir() returns "verus/split/xxx", we need just the verus split path.
    verus_split_path = str(PROJECT_ROOT / module.output_dir())
    report_path = str(HISTORY_DIR / "methodology" / f"{module_name}_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md")
    Path(report_path).parent.mkdir(parents=True, exist_ok=True)

    check_cmd = [
        sys.executable, str(PROJECT_ROOT / "scripts" / "check_spec_methodology.py"),
        verus_split_path, "--module", module.file_stem, "--output", report_path,
    ]
    print(f"[ANALYSIS] Running spec methodology check...")
    result = subprocess.run(check_cmd, capture_output=True, text=True, cwd=str(PROJECT_ROOT))
    print(result.stderr.strip() if result.stderr else "")

    # Read the violations report.
    violations_report = ""
    if Path(report_path).exists():
        violations_report = Path(report_path).read_text()
    if not violations_report or "No methodology violations" in violations_report:
        print("[ANALYSIS] No violations found. Skipping.")
        return True

    prompt = SPEC_METHODOLOGY_PROMPT.format(**fmt, violations_report=violations_report)

    return _run_single_step_with_review(
        module_name=module_name,
        source_path=source_path,
        step_name="spec-methodology",
        prover_prompt=prompt,
        review_prompt_template=SPEC_METHODOLOGY_REVIEW_PROMPT,
        fix_prompt_template=SPEC_METHODOLOGY_FIX_PROMPT,
    )


def run_exec_consistency(module_name: str, source_path: Optional[str] = None) -> bool:
    """
    Check and fix exec code consistency using tree-sitter AST hashing.

    Runs check_exec_consistency.py to detect AST-level differences, then has
    AI fix or document them. Followed by one round of review + fix.
    """
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"ERROR: Could not find source for {module_name}")
            return False

    module = _find_module_config(module_name, source_path)
    fmt = _module_fmt(module)

    print(f"\n{'#'*60}")
    print(f"EXEC CONSISTENCY (tree-sitter): {module_name}")
    print(f"Source: {source_path}")
    print(f"Output: {module.output_dir()}/")
    print(f"{'#'*60}")

    # Step 0: Run tree-sitter consistency check.
    import subprocess
    verus_exec_path = str(PROJECT_ROOT / module.output_dir() / f"{module.file_stem}.rs")
    source_abs = str(PROJECT_ROOT / source_path)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    consistency_report_path = str(HISTORY_DIR / "ast-consistency" / f"{module_name}_{timestamp}.md")
    Path(consistency_report_path).parent.mkdir(parents=True, exist_ok=True)

    # Also generate a fix report path for the AI.
    fix_report_path = str(HISTORY_DIR / "ast-consistency" / f"{module_name}_{timestamp}_fix.md")

    check_cmd = [
        sys.executable, str(PROJECT_ROOT / "scripts" / "check_exec_consistency.py"),
        source_abs, verus_exec_path, "--output", consistency_report_path,
    ]
    print(f"[ANALYSIS] Running tree-sitter AST consistency check...")
    result = subprocess.run(check_cmd, capture_output=True, text=True, cwd=str(PROJECT_ROOT))
    print(result.stderr.strip() if result.stderr else "")

    # Read the consistency report.
    consistency_report = ""
    if Path(consistency_report_path).exists():
        consistency_report = Path(consistency_report_path).read_text()
    if not consistency_report:
        print("[ANALYSIS] Could not generate consistency report.")
        return False

    if "All exec functions consistent" in (result.stderr or ""):
        print("[ANALYSIS] All exec functions consistent. Skipping.")
        return True

    prompt = EXEC_CONSISTENCY_PROMPT.format(
        **fmt,
        consistency_report=consistency_report,
        report_file=fix_report_path,
    )

    return _run_single_step_with_review(
        module_name=module_name,
        source_path=source_path,
        step_name="exec-consistency",
        prover_prompt=prompt,
        review_prompt_template=EXEC_CONSISTENCY_REVIEW_PROMPT,
        fix_prompt_template=EXEC_CONSISTENCY_FIX_PROMPT,
        report_file=fix_report_path,
    )


def run_polish(module_name: str, source_path: Optional[str] = None) -> bool:
    if source_path is None:
        source_path = find_source_path(module_name)
        if source_path is None:
            print(f"ERROR: Could not find original source for {module_name}")
            print("Please specify with --source <path>")
            return False

    print(f"\n{'#'*60}")
    print(f"POLISH PIPELINE: {module_name}")
    print(f"Original: {source_path}")
    print(f"Verified: verus/{module_name}.rs")
    print(f"{'#'*60}")
    print("Steps: 1) Consistency Check  2) Simplify  3) Strengthen Specs")

    # Step 1: Consistency check.
    print(f"\n[STEP 1/3] Consistency Check...")
    if not run_consistency_check(module_name, source_path):
        print("[POLISH] ⚠️ Consistency issues found. Review before continuing.")
        response = input("Continue anyway? [y/N]: ")
        if response.lower() != 'y':
            return False

    # Step 2: Simplify.
    print(f"\n[STEP 2/3] Simplify Proofs...")
    if not run_simplify(module_name, source_path):
        print("[POLISH] ❌ Simplification broke verification!")
        return False

    # Step 3: Strengthen specs.
    print(f"\n[STEP 3/3] Strengthen Specs...")
    if not run_strengthen_liveness(module_name, source_path):
        print("[POLISH] ⚠️ Strengthening failed - some liveness improvements may not verify.")
        # Don't fail the whole pipeline for this.

    print(f"\n{'#'*60}")
    print(f"POLISH COMPLETE: {module_name}")
    print(f"{'#'*60}")

    # Final verification.
    verus_success, _ = run_verus(module_name)
    if verus_success:
        print("[RESULT] ✅ All steps completed, verification passes!")
        git_commit_module(module_name, f"[verus-ai] Polish complete: {module_name}")
        return True
    else:
        print("[RESULT] ❌ Final verification failed!")
        return False


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Verus AI Verification Workflow",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s verify src/kernel/src/mm/phys/upool.rs           # Verify upool
  %(prog)s verify src/libs/bitmap/src/lib.rs --name bitmap  # Verify with custom name
  %(prog)s continue kstack                                  # Continue previous workflow
  %(prog)s continue kstack --rounds 2                       # Continue for 2 more outer rounds
  %(prog)s status                                           # Show verified modules
  %(prog)s check bitmap                                     # Check for cheating

Post-processing commands:
  %(prog)s simplify slab                                    # Remove redundant lemmas/specs
  %(prog)s consistency slab                                 # Check semantic equivalence with original
  %(prog)s consistency slab --source path/to/slab.rs        # Specify original source path
  %(prog)s strengthen slab                                  # Strengthen liveness postconditions
  %(prog)s polish slab                                      # Run all post-processing steps
        """,
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Verify command.
    verify_parser = subparsers.add_parser("verify", help="Verify a source file")
    verify_parser.add_argument("source", help="Source file path")
    verify_parser.add_argument("--name", help="Module name (default: inferred from filename)")
    verify_parser.add_argument("--output-subdir", help="Output subdirectory under verus/split/ (default: module name)")
    verify_parser.add_argument("--file-stem", help="File stem for split files (default: module name)")
    verify_parser.add_argument("--fresh-prover", action="store_true", help="Start new prover session each outer round (default: keep same session)")

    # Continue command.
    continue_parser = subparsers.add_parser("continue", help="Continue a previous workflow")
    continue_parser.add_argument("module", help="Module name to continue")
    continue_parser.add_argument("--rounds", type=int, default=3, help="Maximum additional outer rounds (default: 3)")
    continue_parser.add_argument("--reset", action="store_true", help="Reset to round 1 (start fresh reviews but keep verus code)")
    continue_parser.add_argument("--fresh-prover", action="store_true", help="Start new prover session each outer round")

    # Status command.
    subparsers.add_parser("status", help="Show status of verified modules")

    # Check command.
    check_parser = subparsers.add_parser("check", help="Check for cheating patterns")
    check_parser.add_argument("module", nargs="?", help="Module name (or all if not specified)")

    # Post-processing commands.
    simplify_parser = subparsers.add_parser("simplify", help="Simplify proofs: remove redundant lemmas/specs")
    simplify_parser.add_argument("module", help="Module name to simplify")
    simplify_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    consistency_parser = subparsers.add_parser("consistency", help="Check semantic consistency with original source")
    consistency_parser.add_argument("module", help="Module name to check")
    consistency_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    strengthen_parser = subparsers.add_parser("strengthen", help="Strengthen weak postconditions")
    strengthen_parser.add_argument("module", help="Module name to strengthen")
    strengthen_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    polish_parser = subparsers.add_parser("polish", help="Run full post-processing pipeline (consistency + simplify + strengthen)")
    polish_parser.add_argument("module", help="Module name to polish")
    polish_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    # Improvement commands.
    abstraction_parser = subparsers.add_parser("improve-abstraction", help="Improve spec abstraction with View-level transition functions")
    abstraction_parser.add_argument("module", help="Module name to improve")
    abstraction_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    integrity_parser = subparsers.add_parser("exec-integrity", help="Check exec code integrity against original source")
    integrity_parser.add_argument("module", help="Module name to check")
    integrity_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    methodology_parser = subparsers.add_parser("spec-methodology", help="Check and fix spec methodology per guidelines")
    methodology_parser.add_argument("module", help="Module name to check")
    methodology_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    ast_consistency_parser = subparsers.add_parser("exec-consistency", help="Check exec consistency via tree-sitter AST hashing")
    ast_consistency_parser.add_argument("module", help="Module name to check")
    ast_consistency_parser.add_argument("--source", help="Path to original source file (auto-detected if not specified)")

    args = parser.parse_args()

    if args.command == "verify":
        fresh_prover = getattr(args, 'fresh_prover', False)
        output_subdir = getattr(args, 'output_subdir', None)
        file_stem = getattr(args, 'file_stem', None)
        success = run_workflow(
            args.source,
            module_name=args.name,
            resume=False,
            fresh_prover=fresh_prover,
            output_subdir=output_subdir,
            file_stem=file_stem,
        )
        return 0 if success else 1

    elif args.command == "continue":
        # Continue mode: resume from saved state with additional rounds.
        state = load_workflow_state(args.module)
        if state is None:
            print(f"ERROR: No saved state for module '{args.module}'")
            return 1

        # Reset option: start fresh reviews but keep the verus code.
        if args.reset:
            print(f"\n[RESET] Resetting {args.module} to round 1")
            state.outer_iteration = 1
            state.inner_iteration = 0
            state.current_reviewer_idx = 0
            state.final_grades = {}
            state.completed = False
            save_workflow_state(state)

        # Update max outer iterations for this run.
        import config
        config.MAX_OUTER_ITERATIONS = state.outer_iteration + args.rounds

        fresh_prover = getattr(args, 'fresh_prover', False)
        print(f"\n[CONTINUE] Resuming {args.module} from outer round {state.outer_iteration}")
        print(f"[CONTINUE] Will run up to {args.rounds} more outer rounds (max = {config.MAX_OUTER_ITERATIONS})")
        print(f"[CONTINUE] Prover session: {'fresh each round' if fresh_prover else 'persistent'}")
        if state.final_grades:
            grades_str = ", ".join([f"{k}: {v}" for k, v in state.final_grades.items()])
            print(f"[CONTINUE] Previous grades: {grades_str}")

        # Find the verus file as source.
        verus_file = VERUS_DIR / f"{args.module}.rs"
        if not verus_file.exists():
            print(f"ERROR: Verus file not found: {verus_file}")
            return 1

        success = run_workflow(str(verus_file), module_name=args.module, resume=True, fresh_prover=fresh_prover)
        return 0 if success else 1

    elif args.command == "status":
        print("\nVerified Modules (in verus/):")
        print("-" * 60)
        existing = get_existing_modules()
        for name in existing:
            state = load_workflow_state(name)
            if state:
                status = "COMPLETED" if state.completed else f"Round {state.outer_iteration}"
                grades = state.final_grades or {}
                grades_str = ", ".join([f"{k}:{v}" for k, v in grades.items()]) if grades else "?"
                print(f"  {name:15} {status:20} Grades: {grades_str}")
            else:
                print(f"  {name:15} (no workflow state)")
        if not existing:
            print("  No verified modules found.")
        return 0

    elif args.command == "check":
        if args.module:
            report = detect_cheating_in_module(args.module)
            print(report.detailed_report())
        else:
            from guardrails import detect_cheating_all

            reports = detect_cheating_all()
            for report in reports:
                if report.has_cheating():
                    print(f"\n{report.file_path.name}: {report.summary()}")
        return 0

    elif args.command == "simplify":
        success = run_simplify(args.module, args.source)
        return 0 if success else 1

    elif args.command == "consistency":
        success = run_consistency_check(args.module, args.source)
        return 0 if success else 1

    elif args.command == "strengthen":
        success = run_strengthen_liveness(args.module, args.source)
        return 0 if success else 1

    elif args.command == "polish":
        success = run_polish(args.module, args.source)
        return 0 if success else 1

    elif args.command == "improve-abstraction":
        success = run_improve_abstraction(args.module, getattr(args, 'source', None))
        return 0 if success else 1

    elif args.command == "exec-integrity":
        success = run_exec_integrity(args.module, getattr(args, 'source', None))
        return 0 if success else 1

    elif args.command == "spec-methodology":
        success = run_spec_methodology(args.module, getattr(args, 'source', None))
        return 0 if success else 1

    elif args.command == "exec-consistency":
        success = run_exec_consistency(args.module, getattr(args, 'source', None))
        return 0 if success else 1

    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
