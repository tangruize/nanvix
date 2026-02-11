#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Fix Ghost-in-Exec: Convert ghost types in exec code to concrete types.

This workflow identifies Verus split files where exec code uses Ghost<> types
as struct fields or in function signatures, and converts them to use concrete
types matching the original source code. Ghost state should only appear in
spec/proof files or as ghost parameters/blocks.

Usage:
    python3 verus-ai/scripts/fix_ghost_exec.py [--file FILE] [--dry-run] [--skip-verify]
"""

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import List, Optional, Tuple

# Paths.
PROJECT_ROOT = Path(__file__).parent.parent.parent
VERUS_SPLIT_DIR = PROJECT_ROOT / "verus" / "split"
HISTORY_DIR = PROJECT_ROOT / "verus-ai-history"
LOGS_DIR = HISTORY_DIR / "logs" / "fix_ghost_exec"

# Model to use.
MODEL = "claude-opus-4.6-fast"

# Timeout for copilot (seconds).
COPILOT_TIMEOUT = 2400  # 40 minutes per file.

# Timeout for verus verification (seconds).
VERUS_TIMEOUT = 300  # 5 minutes.


@dataclass
class FileToFix:
    """A file that needs ghost-in-exec fixing."""

    # Path to exec file relative to verus/split/.
    exec_path: str
    # Path to original source file relative to project root.
    source_path: str
    # Verus module path for --verify-module.
    verify_module: str
    # Priority: high, medium, low.
    priority: str
    # Description of what needs fixing.
    description: str


# Files to fix, ordered by priority and dependency.
FILES_TO_FIX: List[FileToFix] = [
    # === MEDIUM: Sync primitives (simpler, good test cases) ===
    FileToFix(
        exec_path="kernel/pm/sync/spinlock.rs",
        source_path="src/kernel/src/pm/sync/spinlock.rs",
        verify_module="kernel::pm::sync::spinlock",
        priority="medium",
        description="Ghost<nat> id and Ghost<bool> token_issued should be concrete (usize, bool).",
    ),
    FileToFix(
        exec_path="kernel/pm/sync/mutex.rs",
        source_path="src/kernel/src/pm/sync/mutex.rs",
        verify_module="kernel::pm::sync::mutex",
        priority="medium",
        description="Ghost<nat> id and Ghost<bool> token_issued should be concrete (usize, bool). "
        "Tracked<MutexToken> returns are OK (linear types pattern).",
    ),
    FileToFix(
        exec_path="kernel/pm/sync/condvar.rs",
        source_path="src/kernel/src/pm/sync/condvar.rs",
        verify_module="kernel::pm::sync::condvar",
        priority="medium",
        description="Ghost<Seq<(int,int)>> sleeping should be concrete Vec or similar. "
        "Oracle Ghost params in remove functions need review.",
    ),
    # === MEDIUM: kcall ===
    FileToFix(
        exec_path="kernel/kcall/scoreboard.rs",
        source_path="src/kernel/src/kcall/scoreboard.rs",
        verify_module="kernel::kcall::scoreboard",
        priority="medium",
        description="Ghost<nat> completed_cycles should be concrete (usize or u64).",
    ),
    FileToFix(
        exec_path="kernel/kcall/handler.rs",
        source_path="src/kernel/src/kcall/handler.rs",
        verify_module="kernel::kcall::handler",
        priority="medium",
        description="Ghost<HarvestOutcome> fields in result structs should be concrete.",
    ),
    # === HIGH: Process state (complex, interdependent) ===
    FileToFix(
        exec_path="kernel/pm/process/state/process_state.rs",
        source_path="src/kernel/src/pm/process/state/mod.rs",
        verify_module="kernel::pm::process::state::process_state",
        priority="high",
        description="Ghost<Map> mutexes/conditions and Ghost<Seq> pmio should be concrete. "
        "ProcessState wraps the PID and capability data.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/state/zombie.rs",
        source_path="src/kernel/src/pm/process/state/zombie.rs",
        verify_module="kernel::pm::process::state::zombie",
        priority="high",
        description="Ghost<int> pid, Ghost<Seq<int>> zombie_thread_ids, Ghost<int> status "
        "should all be concrete types.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/state/sleeping.rs",
        source_path="src/kernel/src/pm/process/state/sleeping.rs",
        verify_module="kernel::pm::process::state::sleeping",
        priority="high",
        description="All Ghost struct fields (pid, thread ID lists) should be concrete.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/state/interrupted.rs",
        source_path="src/kernel/src/pm/process/state/interrupted.rs",
        verify_module="kernel::pm::process::state::interrupted",
        priority="high",
        description="All Ghost struct fields should be concrete. InterruptedProcess and "
        "RunnableProcess boundary models need concrete types.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/state/runnable.rs",
        source_path="src/kernel/src/pm/process/state/runnable.rs",
        verify_module="kernel::pm::process::state::runnable",
        priority="high",
        description="18 Ghost fields across 4 structs should all be concrete types.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/state/running.rs",
        source_path="src/kernel/src/pm/process/state/running.rs",
        verify_module="kernel::pm::process::state::running",
        priority="high",
        description="21 Ghost fields across 5 structs. Most complex file. "
        "All should be concrete types matching original.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/manager/process_manager.rs",
        source_path="src/kernel/src/pm/process/manager/mod.rs",
        verify_module="kernel::pm::process::manager::process_manager",
        priority="high",
        description="4 Ghost<Set<int>> fields (ready, suspended, interrupted, zombies) "
        "should be concrete sets or equivalent.",
    ),
    FileToFix(
        exec_path="kernel/pm/process/manager/process_manager_unsafe.rs",
        source_path="src/kernel/src/pm/process/manager/unsafe.rs",
        verify_module="kernel::pm::process::manager::process_manager_unsafe",
        priority="high",
        description="Ghost<bool> ghost_diverged should be concrete bool.",
    ),
]


def count_ghost_in_exec(filepath: Path) -> int:
    """Count Ghost<>/Tracked<> occurrences in an exec file."""
    if not filepath.exists():
        return 0
    content = filepath.read_text()
    # Count Ghost< and Tracked< in struct field definitions.
    ghost_count = len(re.findall(r"pub\s+\w+:\s*Ghost<", content))
    return ghost_count


def run_verus_verify(module: str, timeout: int = VERUS_TIMEOUT) -> Tuple[bool, str]:
    """Run verus verification for a specific module."""
    cmd = ["verus", "--crate-type", "lib", "lib.rs", "--verify-module", module]
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(VERUS_SPLIT_DIR),
        )
        output = result.stdout + result.stderr
        success = result.returncode == 0 and "0 errors" in output
        return success, output
    except subprocess.TimeoutExpired:
        return False, f"TIMEOUT after {timeout}s"
    except Exception as e:
        return False, f"ERROR: {e}"


def run_verus_full(timeout: int = 600) -> Tuple[bool, str]:
    """Run full verus verification."""
    cmd = ["verus", "--crate-type", "lib", "lib.rs"]
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(VERUS_SPLIT_DIR),
        )
        output = result.stdout + result.stderr
        success = result.returncode == 0 and "0 errors" in output
        return success, output
    except subprocess.TimeoutExpired:
        return False, f"TIMEOUT after {timeout}s"
    except Exception as e:
        return False, f"ERROR: {e}"


def build_prompt(file_to_fix: FileToFix) -> str:
    """Build the prompt for fixing a single file."""
    exec_path = VERUS_SPLIT_DIR / file_to_fix.exec_path
    source_path = PROJECT_ROOT / file_to_fix.source_path

    # Derive spec and proof file paths.
    stem = exec_path.stem
    spec_path = exec_path.parent / f"{stem}.spec.rs"
    proof_path = exec_path.parent / f"{stem}.proof.rs"

    exec_content = exec_path.read_text() if exec_path.exists() else "(FILE NOT FOUND)"
    source_content = source_path.read_text() if source_path.exists() else "(FILE NOT FOUND)"
    spec_content = spec_path.read_text() if spec_path.exists() else "(FILE NOT FOUND)"
    proof_content = proof_path.read_text() if proof_path.exists() else "(FILE NOT FOUND)"

    prompt = f"""Fix Ghost-in-Exec issue in Verus verification file.

## PROBLEM
The exec file `verus/split/{file_to_fix.exec_path}` uses `Ghost<>` types as struct fields
and in exec function signatures. This is WRONG because:
- Exec code should use CONCRETE types matching the original source code
- Ghost<> types are erased at compile time - they don't verify real executable behavior
- Ghost<> should ONLY appear in: ghost parameters, ghost blocks, spec/proof files

## SPECIFIC ISSUES IN THIS FILE
{file_to_fix.description}

## WHAT TO DO
1. Read the ORIGINAL source code below to understand the concrete types
2. Read the CURRENT exec/spec/proof files below
3. Modify the exec file (`verus/split/{file_to_fix.exec_path}`) to:
   - Replace Ghost<> struct fields with CONCRETE types matching the original
   - For types Verus can't handle directly (e.g., Arc, AtomicBool, complex enums),
     use a simplified but CONCRETE representation (e.g., bool instead of AtomicBool,
     u64 instead of Arc<...>, Vec<u64> instead of Option<NonEmptyVecDeque<Thread>>)
   - Keep Ghost<> parameters that are purely for proof purposes (e.g., Ghost<CallerContext>)
   - Move any abstract state that was in Ghost struct fields to the View type in spec.rs
4. Update the spec file (`verus/split/{file_to_fix.exec_path.replace('.rs', '.spec.rs')}`) as needed:
   - The View type should map concrete exec fields to abstract spec state
   - spec functions should reference the View type
5. Update the proof file (`verus/split/{file_to_fix.exec_path.replace('.rs', '.proof.rs')}`) as needed:
   - Lemmas may need updates to work with the new concrete types
6. PRESERVE all verification strength - do NOT weaken requires/ensures

## CRITICAL RULES
- Do NOT use `assume()` or add new `#[verifier::external_body]` annotations
- Do NOT remove existing verified properties
- Do NOT simplify ensures clauses just to make conversion easier
- Ghost<> as FUNCTION PARAMETERS for proof purposes is OK (standard Verus pattern)
- Tracked<Token> patterns are OK (Verus linear types)
- ALL struct fields in exec code MUST be concrete (not Ghost<>)
- If a field was Ghost<int>, make it a concrete integer type (u64, i64, usize, etc.)
- If a field was Ghost<Seq<int>>, make it a concrete Vec<u64> or similar
- If a field was Ghost<bool>, make it a concrete bool
- If a field was Ghost<nat>, make it a concrete usize or u64
- If a field was Ghost<Map<K,V>>, consider a concrete representation
- If a field was Ghost<Set<int>>, consider a concrete representation

## VERIFICATION
After making changes, verify with:
```
cd verus/split && verus --crate-type lib lib.rs --verify-module {file_to_fix.verify_module}
```
Iterate until verification passes with 0 errors.

## ORIGINAL SOURCE CODE ({file_to_fix.source_path})
```rust
{source_content}
```

## CURRENT EXEC FILE (verus/split/{file_to_fix.exec_path})
```rust
{exec_content}
```

## CURRENT SPEC FILE (verus/split/{file_to_fix.exec_path.replace('.rs', '.spec.rs')})
```rust
{spec_content}
```

## CURRENT PROOF FILE (verus/split/{file_to_fix.exec_path.replace('.rs', '.proof.rs')})
```rust
{proof_content}
```
"""
    return prompt


def build_review_prompt(file_to_fix: FileToFix) -> str:
    """Build a review prompt to check the fix quality."""
    exec_path = VERUS_SPLIT_DIR / file_to_fix.exec_path
    source_path = PROJECT_ROOT / file_to_fix.source_path
    stem = exec_path.stem
    spec_path = exec_path.parent / f"{stem}.spec.rs"
    proof_path = exec_path.parent / f"{stem}.proof.rs"

    exec_content = exec_path.read_text() if exec_path.exists() else "(FILE NOT FOUND)"
    source_content = source_path.read_text() if source_path.exists() else "(FILE NOT FOUND)"
    spec_content = spec_path.read_text() if spec_path.exists() else "(FILE NOT FOUND)"
    proof_content = proof_path.read_text() if proof_path.exists() else "(FILE NOT FOUND)"

    prompt = f"""Review the Ghost-in-Exec fix for `verus/split/{file_to_fix.exec_path}`.

## REVIEW CRITERIA
1. Are ALL struct fields in the exec file concrete (not Ghost<>)?
   - Ghost<> as function parameters for proof is OK
   - Tracked<Token> patterns are OK
   - But Ghost<> as STRUCT FIELDS is NOT OK
2. Do the concrete types match the original source semantics?
3. Are verification properties preserved (not weakened)?
4. Does it pass verus verification?
5. Were any `assume()` or `external_body` annotations added?

## FILES TO REVIEW

### Original Source ({file_to_fix.source_path})
```rust
{source_content}
```

### Exec File (verus/split/{file_to_fix.exec_path})
```rust
{exec_content}
```

### Spec File
```rust
{spec_content}
```

### Proof File
```rust
{proof_content}
```

## INSTRUCTIONS
1. Check if there are remaining Ghost<> struct fields in the exec file
2. If there are issues, FIX them directly (edit the files)
3. Verify with: cd verus/split && verus --crate-type lib lib.rs --verify-module {file_to_fix.verify_module}
4. Report what you found and fixed

Output format:
- REMAINING_GHOST_FIELDS: count
- ISSUES_FOUND: list
- FIXES_APPLIED: list
- VERIFICATION: PASS/FAIL
"""
    return prompt


def run_copilot(prompt: str, log_file: Path, timeout: int = COPILOT_TIMEOUT) -> Tuple[bool, str]:
    """Run copilot CLI with a prompt."""
    cmd = [
        "copilot",
        "--allow-all-tools",
        "--allow-all-paths",
        "--model", MODEL,
        "-p", prompt,
    ]

    log_file.parent.mkdir(parents=True, exist_ok=True)

    # Write prompt to log.
    with open(log_file, "w") as f:
        f.write(f"=== Copilot Session ===\n")
        f.write(f"Timestamp: {datetime.now().isoformat()}\n")
        f.write(f"Model: {MODEL}\n")
        f.write(f"\n=== Prompt ===\n")
        f.write(prompt[:2000] + "\n... (truncated)\n")
        f.write(f"\n=== Output ===\n")
        f.flush()

    start_time = datetime.now()
    output_lines = []

    try:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            cwd=str(PROJECT_ROOT),
            bufsize=1,
        )

        with open(log_file, "a") as f:
            for line in process.stdout:
                f.write(line)
                f.flush()
                output_lines.append(line)
                # Print progress indicator.
                if "verified" in line.lower() or "error" in line.lower():
                    print(f"  >> {line.strip()}")

        process.wait(timeout=timeout)
        exit_code = process.returncode
        output = "".join(output_lines)

    except subprocess.TimeoutExpired:
        process.kill()
        output = "".join(output_lines) + f"\nTIMEOUT after {timeout}s"
        exit_code = 124
    except Exception as e:
        output = "".join(output_lines) + f"\nERROR: {e}"
        exit_code = 1

    duration = (datetime.now() - start_time).total_seconds()

    with open(log_file, "a") as f:
        f.write(f"\n=== End ===\n")
        f.write(f"Duration: {duration:.1f}s\n")
        f.write(f"Exit code: {exit_code}\n")

    print(f"  Duration: {duration:.1f}s, exit code: {exit_code}")
    return exit_code == 0, output


def git_commit(message: str) -> bool:
    """Commit verus changes."""
    try:
        subprocess.run(["git", "add", "verus/"], cwd=str(PROJECT_ROOT), check=True,
                       capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", message],
            cwd=str(PROJECT_ROOT), check=True, capture_output=True,
        )
        return True
    except subprocess.CalledProcessError:
        return False


def process_file(file_to_fix: FileToFix, dry_run: bool = False,
                 skip_verify: bool = False) -> dict:
    """Process a single file: fix ghost-in-exec, verify, optionally review."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    module_short = file_to_fix.exec_path.replace("/", "_").replace(".rs", "")

    result = {
        "file": file_to_fix.exec_path,
        "module": file_to_fix.verify_module,
        "priority": file_to_fix.priority,
        "ghost_before": 0,
        "ghost_after": 0,
        "prover_success": False,
        "verify_success": False,
        "review_done": False,
        "final_verify": False,
    }

    exec_path = VERUS_SPLIT_DIR / file_to_fix.exec_path

    # Count ghost fields before.
    result["ghost_before"] = count_ghost_in_exec(exec_path)
    print(f"\n{'='*60}")
    print(f"Processing: {file_to_fix.exec_path}")
    print(f"Priority: {file_to_fix.priority}")
    print(f"Ghost struct fields before: {result['ghost_before']}")
    print(f"{'='*60}")

    if result["ghost_before"] == 0:
        print("  No ghost struct fields found, skipping.")
        result["prover_success"] = True
        result["verify_success"] = True
        result["final_verify"] = True
        return result

    if dry_run:
        print("  [DRY RUN] Would fix this file.")
        return result

    # Step 1: Run prover (fix ghost-in-exec).
    print("\n  Step 1: Running prover to fix ghost-in-exec...")
    prompt = build_prompt(file_to_fix)
    log_file = LOGS_DIR / module_short / f"prover_{timestamp}.txt"
    success, output = run_copilot(prompt, log_file)
    result["prover_success"] = success

    # Count ghost fields after.
    result["ghost_after"] = count_ghost_in_exec(exec_path)
    print(f"  Ghost struct fields after: {result['ghost_after']}")

    # Step 2: Verify.
    if not skip_verify:
        print("\n  Step 2: Verifying module...")
        verify_ok, verify_output = run_verus_verify(file_to_fix.verify_module)
        result["verify_success"] = verify_ok

        # Extract stats.
        match = re.search(r"(\d+) verified, (\d+) errors", verify_output)
        if match:
            print(f"  Verification: {match.group(1)} verified, {match.group(2)} errors")
        else:
            print(f"  Verification: {'PASS' if verify_ok else 'FAIL'}")

        if not verify_ok:
            # Save verification error log.
            err_log = LOGS_DIR / module_short / f"verify_error_{timestamp}.txt"
            err_log.parent.mkdir(parents=True, exist_ok=True)
            err_log.write_text(verify_output)
            print(f"  Error log saved: {err_log}")

    # Step 3: Review (1 round max) - only if there are still ghost fields.
    if result["ghost_after"] > 0:
        print("\n  Step 3: Running reviewer to check remaining issues...")
        review_prompt = build_review_prompt(file_to_fix)
        review_log = LOGS_DIR / module_short / f"review_{timestamp}.txt"
        review_success, review_output = run_copilot(review_prompt, review_log)
        result["review_done"] = True

        # Re-count after review.
        result["ghost_after"] = count_ghost_in_exec(exec_path)
        print(f"  Ghost struct fields after review: {result['ghost_after']}")

        # Re-verify after review.
        if not skip_verify:
            verify_ok, verify_output = run_verus_verify(file_to_fix.verify_module)
            result["final_verify"] = verify_ok
            match = re.search(r"(\d+) verified, (\d+) errors", verify_output)
            if match:
                print(f"  Final verification: {match.group(1)} verified, {match.group(2)} errors")
    else:
        result["final_verify"] = result["verify_success"]

    # Commit.
    status = "fixed" if result["ghost_after"] == 0 else "partially-fixed"
    commit_msg = (
        f"[verus] fix-ghost-exec: {status} {file_to_fix.exec_path} "
        f"(ghost: {result['ghost_before']}->{result['ghost_after']})"
    )
    git_commit(commit_msg)

    return result


def main():
    parser = argparse.ArgumentParser(description="Fix Ghost-in-Exec in Verus split files")
    parser.add_argument("--file", type=str, help="Fix only a specific file (exec path)")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done")
    parser.add_argument("--skip-verify", action="store_true", help="Skip verus verification")
    parser.add_argument("--priority", type=str, choices=["high", "medium", "low", "all"],
                        default="all", help="Only process files of this priority")
    args = parser.parse_args()

    LOGS_DIR.mkdir(parents=True, exist_ok=True)

    # Filter files.
    files = FILES_TO_FIX
    if args.file:
        files = [f for f in files if args.file in f.exec_path]
        if not files:
            print(f"No files match: {args.file}")
            sys.exit(1)
    if args.priority != "all":
        files = [f for f in files if f.priority == args.priority]

    print(f"{'='*60}")
    print(f"Fix Ghost-in-Exec Workflow")
    print(f"Files to process: {len(files)}")
    print(f"Model: {MODEL}")
    print(f"{'='*60}")

    # Check baseline.
    if not args.dry_run and not args.skip_verify:
        print("\nChecking baseline verification...")
        ok, output = run_verus_full()
        match = re.search(r"(\d+) verified, (\d+) errors", output)
        if match:
            print(f"Baseline: {match.group(1)} verified, {match.group(2)} errors")
        if not ok:
            print("WARNING: Baseline verification fails! Proceeding anyway.")

    results = []
    for file_to_fix in files:
        result = process_file(file_to_fix, dry_run=args.dry_run,
                              skip_verify=args.skip_verify)
        results.append(result)

    # Final full verification.
    if not args.dry_run and not args.skip_verify:
        print(f"\n{'='*60}")
        print("Running final full verification...")
        print(f"{'='*60}")
        ok, output = run_verus_full(timeout=600)
        match = re.search(r"(\d+) verified, (\d+) errors", output)
        if match:
            print(f"Final: {match.group(1)} verified, {match.group(2)} errors")
        else:
            print(f"Final: {'PASS' if ok else 'FAIL'}")

    # Summary.
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    total_ghost_before = sum(r["ghost_before"] for r in results)
    total_ghost_after = sum(r["ghost_after"] for r in results)
    fully_fixed = sum(1 for r in results if r["ghost_after"] == 0 and r["ghost_before"] > 0)
    partially_fixed = sum(1 for r in results if 0 < r["ghost_after"] < r["ghost_before"])
    failed = sum(1 for r in results if r["ghost_after"] >= r["ghost_before"] and r["ghost_before"] > 0)
    verify_pass = sum(1 for r in results if r.get("final_verify") or r.get("verify_success"))

    print(f"Total ghost struct fields: {total_ghost_before} -> {total_ghost_after}")
    print(f"Fully fixed: {fully_fixed}")
    print(f"Partially fixed: {partially_fixed}")
    print(f"Not fixed: {failed}")
    print(f"Verification passing: {verify_pass}/{len(results)}")
    print()

    for r in results:
        status = "✓" if r["ghost_after"] == 0 else "⚠" if r["ghost_after"] < r["ghost_before"] else "✗"
        verify = "✓" if r.get("final_verify") or r.get("verify_success") else "✗"
        print(f"  {status} {r['file']}: ghost {r['ghost_before']}->{r['ghost_after']} verify:{verify}")

    # Save summary report.
    summary_file = LOGS_DIR / f"summary_{datetime.now().strftime('%Y%m%d_%H%M%S')}.txt"
    with open(summary_file, "w") as f:
        f.write("Fix Ghost-in-Exec Summary\n")
        f.write(f"Date: {datetime.now().isoformat()}\n")
        f.write(f"Total ghost: {total_ghost_before} -> {total_ghost_after}\n")
        f.write(f"Fixed: {fully_fixed}, Partial: {partially_fixed}, Failed: {failed}\n\n")
        for r in results:
            f.write(f"{r['file']}: {r['ghost_before']}->{r['ghost_after']} "
                    f"verify:{'pass' if r.get('final_verify') or r.get('verify_success') else 'fail'}\n")

    print(f"\nSummary saved: {summary_file}")


if __name__ == "__main__":
    main()
