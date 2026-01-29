# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Configuration for the Verus AI verification workflow.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional

# Model configurations.
PROVER_MODEL = "claude-opus-4.5"

REVIEWER_MODELS = [
    "claude-opus-4.5",
    "gpt-5.1-codex-max",
    "gemini-3-pro-preview",
]

# Directory paths.
PROJECT_ROOT = Path(__file__).parent.parent
VERUS_DIR = PROJECT_ROOT / "verus"
HISTORY_DIR = PROJECT_ROOT / "verus-ai-history"
LOGS_DIR = HISTORY_DIR / "logs"
REVIEWS_DIR = HISTORY_DIR / "reviews"

# Verification command.
VERUS_CMD = "verus --crate-type lib lib.rs"

# Timeout settings (in seconds).
PROVER_TIMEOUT = 1800  # 30 minutes for prover.
REVIEWER_TIMEOUT = 900  # 15 minutes for reviewer.

# Review iteration limits.
MAX_OUTER_ITERATIONS = 3  # Max rounds where all reviewers go through.
MAX_INNER_ITERATIONS = 3  # Max fix attempts per reviewer within one round.
GRADE_THRESHOLD = "A"  # Minimum grade to accept (A+, A, A-, B+, etc.).

# Initial prover retry limit.
INITIAL_PROVER_RETRIES = 3  # Max retries if Verus fails during initial generation.

# Workflow mode: "serial" (one-on-one with each reviewer).
WORKFLOW_MODE = "serial"

# Cheating detection patterns.
CHEATING_PATTERNS = {
    "assume": r"\bassume\s*\(",
    "external_body": r"#\s*\[\s*verifier\s*::\s*external_body\s*\]",
    "admit": r"\badmit\s*\(",
    "trusted": r"#\s*\[\s*verifier\s*::\s*trusted\s*\]",
}


@dataclass
class ModuleConfig:
    """Configuration for a module to verify."""

    name: str
    source_path: Path
    verify_module: str = ""  # If empty, verifies entire crate.

    def verus_cmd(self) -> str:
        """Return the Verus command for this module."""
        if self.verify_module:
            return f"{VERUS_CMD} --verify-module {self.verify_module}"
        return VERUS_CMD

    @classmethod
    def from_source_path(cls, source_path: str, module_name: Optional[str] = None) -> "ModuleConfig":
        """
        Create a ModuleConfig from a source file path.

        Parameters:
            source_path: Path to the source file (relative to PROJECT_ROOT).
            module_name: Optional module name. If not provided, inferred from filename.

        Returns:
            ModuleConfig instance.
        """
        path = Path(source_path)

        # Infer module name from filename if not provided.
        if module_name is None:
            # e.g., "lib.rs" -> use parent dir name, "kpool.rs" -> "kpool"
            if path.stem == "lib":
                module_name = path.parent.name
            else:
                module_name = path.stem

        return cls(
            name=module_name,
            source_path=path,
            verify_module=module_name,
        )


def get_existing_modules() -> List[str]:
    """
    Get list of already verified modules from verus/ directory.

    Returns:
        List of module names (without .rs extension).
    """
    if not VERUS_DIR.exists():
        return []

    modules = []
    for rs_file in VERUS_DIR.glob("*.rs"):
        if rs_file.name != "lib.rs":
            modules.append(rs_file.stem)
    return sorted(modules)
