# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Cheating detection and guardrails for the Verus AI verification workflow.

Supports both legacy single-file modules (verus/{name}.rs) and the three-file
split organization (verus/split/{subdir}/{stem}.rs + .spec.rs + .proof.rs).
"""

import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Tuple

from config import CHEATING_PATTERNS, PROJECT_ROOT, VERUS_DIR, VERUS_SPLIT_DIR


@dataclass
class CheatingReport:
    """Report of detected cheating patterns."""

    file_path: Path
    assume_count: int = 0
    external_body_count: int = 0
    admit_count: int = 0
    trusted_count: int = 0
    locations: Dict[str, List[int]] = field(default_factory=dict)

    def has_cheating(self) -> bool:
        """Check if any cheating patterns were detected."""
        return (
            self.assume_count > 0
            or self.external_body_count > 0
            or self.admit_count > 0
            or self.trusted_count > 0
        )

    def summary(self) -> str:
        """Return a summary of detected patterns."""
        parts = []
        if self.assume_count > 0:
            parts.append(f"assume: {self.assume_count}")
        if self.external_body_count > 0:
            parts.append(f"external_body: {self.external_body_count}")
        if self.admit_count > 0:
            parts.append(f"admit: {self.admit_count}")
        if self.trusted_count > 0:
            parts.append(f"trusted: {self.trusted_count}")
        return ", ".join(parts) if parts else "No cheating detected"

    def detailed_report(self) -> str:
        """Return a detailed report with line numbers."""
        lines = [f"# Cheating Detection Report: {self.file_path.name}", ""]
        lines.append(f"## Summary: {self.summary()}")
        lines.append("")

        for pattern_name, line_numbers in self.locations.items():
            if line_numbers:
                lines.append(f"### {pattern_name}")
                for line_num in line_numbers:
                    lines.append(f"- Line {line_num}")
                lines.append("")

        return "\n".join(lines)


def detect_cheating(file_path: Path) -> CheatingReport:
    """
    Detect cheating patterns in a Verus file.

    Parameters:
        file_path: Path to the Verus file.

    Returns:
        CheatingReport with detection results.
    """
    report = CheatingReport(file_path=file_path)

    if not file_path.exists():
        return report

    content = file_path.read_text()
    lines = content.split("\n")

    for pattern_name, pattern in CHEATING_PATTERNS.items():
        regex = re.compile(pattern, re.IGNORECASE)
        line_numbers = []

        for i, line in enumerate(lines, 1):
            if regex.search(line):
                line_numbers.append(i)

        if line_numbers:
            report.locations[pattern_name] = line_numbers

            if pattern_name == "assume":
                report.assume_count = len(line_numbers)
            elif pattern_name == "external_body":
                report.external_body_count = len(line_numbers)
            elif pattern_name == "admit":
                report.admit_count = len(line_numbers)
            elif pattern_name == "trusted":
                report.trusted_count = len(line_numbers)

    return report


def _find_module_files(module_name: str) -> List[Path]:
    """
    Find all files belonging to a module (split or legacy).

    Parameters:
        module_name: Name of the module.

    Returns:
        List of file paths to scan.
    """
    files = []

    # Check split directory: search for {stem}.rs, {stem}.spec.rs, {stem}.proof.rs.
    if VERUS_SPLIT_DIR.exists():
        for rs_file in VERUS_SPLIT_DIR.rglob(f"{module_name}.rs"):
            files.append(rs_file)
        for rs_file in VERUS_SPLIT_DIR.rglob(f"{module_name}.spec.rs"):
            files.append(rs_file)
        for rs_file in VERUS_SPLIT_DIR.rglob(f"{module_name}.proof.rs"):
            files.append(rs_file)
        # Also check lib.rs pattern (e.g., libs/bitmap/lib.rs).
        for rs_file in VERUS_SPLIT_DIR.rglob("lib.rs"):
            if rs_file.parent.name == module_name:
                files.append(rs_file)
                spec_file = rs_file.parent / "lib.spec.rs"
                proof_file = rs_file.parent / "lib.proof.rs"
                if spec_file.exists():
                    files.append(spec_file)
                if proof_file.exists():
                    files.append(proof_file)

    # Check legacy single-file.
    legacy_file = VERUS_DIR / f"{module_name}.rs"
    if legacy_file.exists():
        files.append(legacy_file)

    return files


def detect_cheating_in_module(module_name: str) -> CheatingReport:
    """
    Detect cheating patterns in a module (supports split files).

    Parameters:
        module_name: Name of the module.

    Returns:
        CheatingReport with aggregated detection results.
    """
    files = _find_module_files(module_name)

    if not files:
        # Fall back to legacy path.
        file_path = VERUS_DIR / f"{module_name}.rs"
        return detect_cheating(file_path)

    # Aggregate reports from all files.
    combined = CheatingReport(file_path=files[0])
    for f in files:
        report = detect_cheating(f)
        combined.assume_count += report.assume_count
        combined.external_body_count += report.external_body_count
        combined.admit_count += report.admit_count
        combined.trusted_count += report.trusted_count
        for pattern_name, line_numbers in report.locations.items():
            prefixed = [f"{f.name}:{ln}" for ln in line_numbers]
            if pattern_name not in combined.locations:
                combined.locations[pattern_name] = []
            combined.locations[pattern_name].extend(line_numbers)

    return combined


def detect_cheating_all() -> List[CheatingReport]:
    """
    Detect cheating patterns in all Verus files.

    Returns:
        List of CheatingReports.
    """
    reports = []

    # Check split directory.
    if VERUS_SPLIT_DIR.exists():
        for rs_file in VERUS_SPLIT_DIR.rglob("*.rs"):
            if rs_file.name in ("lib.rs", "mod.rs"):
                continue
            reports.append(detect_cheating(rs_file))

    # Check legacy directory.
    for rs_file in VERUS_DIR.glob("*.rs"):
        if rs_file.name != "lib.rs":
            reports.append(detect_cheating(rs_file))

    return reports


def run_verus(module_name: str = "", timeout: int = 120, use_script: bool = True) -> Tuple[bool, str]:
    """
    Run Verus verification.

    Parameters:
        module_name: Optional module name to verify.
        timeout: Timeout in seconds.
        use_script: If True, use verify.sh script which auto-commits.

    Returns:
        Tuple of (success, output).
    """
    if use_script:
        # Use the verify.sh script which handles logging and git commits.
        script_path = PROJECT_ROOT / "verus-ai" / "scripts" / "verify.sh"
        cmd = ["bash", str(script_path)]
        if module_name:
            cmd.append(module_name)
        cwd = PROJECT_ROOT
    else:
        cmd = ["verus", "--crate-type", "lib", "lib.rs"]
        if module_name:
            cmd.extend(["--verify-module", module_name])
        cwd = VERUS_SPLIT_DIR

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=cwd,
        )
        output = result.stdout + result.stderr
        success = result.returncode == 0
        # Also check for "0 errors" in output.
        if "verification results:" in output:
            success = success and "0 errors" in output
        return success, output
    except subprocess.TimeoutExpired:
        return False, f"ERROR: Verification timed out after {timeout} seconds"
    except Exception as e:
        return False, f"ERROR: {e}"


def git_commit_verus(message: str) -> bool:
    """
    Commit all changes in verus/ directory and verus-ai-history/.

    Parameters:
        message: Commit message.

    Returns:
        True if successful.
    """
    try:
        # Stage verus directory.
        subprocess.run(
            ["git", "add", "verus/"],
            cwd=PROJECT_ROOT,
            check=False,
            capture_output=True,
        )

        # Stage history directory.
        subprocess.run(
            ["git", "add", "verus-ai-history/"],
            cwd=PROJECT_ROOT,
            check=False,
            capture_output=True,
        )

        # Check if there's anything to commit.
        result = subprocess.run(
            ["git", "diff", "--cached", "--quiet"],
            cwd=PROJECT_ROOT,
            capture_output=True,
        )
        if result.returncode == 0:
            # Nothing to commit.
            return False

        # Commit with message.
        subprocess.run(
            ["git", "commit", "-m", message],
            cwd=PROJECT_ROOT,
            check=True,
            capture_output=True,
        )
        return True
    except subprocess.CalledProcessError:
        return False


def git_commit_module(module_name: str, message: str) -> bool:
    """
    Commit changes to a module (legacy wrapper).

    Parameters:
        module_name: Name of the module.
        message: Commit message.

    Returns:
        True if successful.
    """
    return git_commit_verus(message)


def git_diff_module(module_name: str) -> str:
    """
    Get git diff for a module.

    Parameters:
        module_name: Name of the module.

    Returns:
        Diff string.
    """
    # Try split directory first.
    files = _find_module_files(module_name)
    if files:
        diffs = []
        for f in files:
            try:
                result = subprocess.run(
                    ["git", "diff", str(f)],
                    cwd=PROJECT_ROOT,
                    capture_output=True,
                    text=True,
                )
                if result.stdout:
                    diffs.append(result.stdout)
            except Exception:
                pass
        return "\n".join(diffs)

    # Fall back to legacy.
    file_path = VERUS_DIR / f"{module_name}.rs"
    if not file_path.exists():
        return ""

    try:
        result = subprocess.run(
            ["git", "diff", str(file_path)],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True,
        )
        return result.stdout
    except Exception:
        return ""
