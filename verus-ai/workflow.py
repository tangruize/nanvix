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
    GRADE_THRESHOLD,
    HISTORY_DIR,
    INITIAL_PROVER_RETRIES,
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
    PROVER_FIX_PROMPT,
    PROVER_FIX_FRESH_PROMPT,
    PROVER_PROMPT,
    PROVER_RETRY_PROMPT,
    REVIEW_FOLLOWUP_PROMPT,
    REVIEWER_PROMPT,
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

    prompt = PROVER_PROMPT.format(
        source_path=module.source_path,
        module_name=module.name,
        verus_cmd=module.verus_cmd(),
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
            module_name=module.name,
            retry_num=retry_num,
            max_retries=INITIAL_PROVER_RETRIES,
            verus_output=verus_output[:2000],  # Truncate to avoid token overflow.
            verus_cmd=module.verus_cmd(),
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
    model_short = model.split("-")[0]  # e.g., "claude" from "claude-opus-4.5"
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
            review_file=review_file,
            result_file=result_file,
        )
    else:
        # Initial review (new reviewer or new outer round).
        prompt = REVIEWER_PROMPT.format(
            module_name=module.name,
            source_path=module.source_path,
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
            dependencies=deps_str,
        )
    else:
        prompt = PROVER_FIX_PROMPT.format(
            review_file=review_refs,
            module_name=module.name,
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

    Returns:
        True if verification succeeded (all reviewers passed).
    """
    # Create module config.
    module = ModuleConfig.from_source_path(source_path, module_name)

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
        """,
    )

    subparsers = parser.add_subparsers(dest="command", help="Command to run")

    # Verify command.
    verify_parser = subparsers.add_parser("verify", help="Verify a source file")
    verify_parser.add_argument("source", help="Source file path")
    verify_parser.add_argument("--name", help="Module name (default: inferred from filename)")
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

    args = parser.parse_args()

    if args.command == "verify":
        fresh_prover = getattr(args, 'fresh_prover', False)
        success = run_workflow(args.source, module_name=args.name, resume=False, fresh_prover=fresh_prover)
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

    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
