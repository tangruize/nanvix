# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Configuration for the Verus AI verification workflow.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import List, Optional

# Model configurations.
PROVER_MODEL = "claude-opus-4.6-fast"

REVIEWER_MODELS = [
    "claude-opus-4.6",
    "gpt-5.2-codex",
    "gemini-3-pro-preview",
]

# Directory paths.
PROJECT_ROOT = Path(__file__).parent.parent
VERUS_DIR = PROJECT_ROOT / "verus"
VERUS_SPLIT_DIR = VERUS_DIR / "split"
HISTORY_DIR = PROJECT_ROOT / "verus-ai-history"
LOGS_DIR = HISTORY_DIR / "logs"
REVIEWS_DIR = HISTORY_DIR / "reviews"
CONSISTENCY_DIR = HISTORY_DIR / "consistency"
SIMPLIFY_DIR = HISTORY_DIR / "simplify"
STRENGTHEN_DIR = HISTORY_DIR / "strengthen"
INTEGRITY_DIR = HISTORY_DIR / "integrity"

# Tree-sitter Python executable (venv with tree_sitter==0.21.3).
TREE_SITTER_PYTHON = Path("/tmp/verus-tools-venv/bin/python3")

# Verification command (run from VERUS_SPLIT_DIR).
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
    """Configuration for a module to verify.

    Supports the three-file split organization where output goes to a
    subdirectory under verus/split/ with separate .rs, .spec.rs, .proof.rs files.
    """

    name: str
    source_path: Path
    verify_module: str = ""  # If empty, verifies entire crate.
    # Relative output directory under verus/split/ (e.g., "kernel/pm/thread").
    output_subdir: str = ""
    # File stem for the three-file split (e.g., "state" -> state.rs, state.spec.rs, state.proof.rs).
    file_stem: str = ""

    def verus_cmd(self) -> str:
        """Return the Verus command for this module."""
        if self.verify_module:
            return f"{VERUS_CMD} --verify-module {self.verify_module}"
        return VERUS_CMD

    def output_dir(self) -> str:
        """Return the output directory path relative to project root."""
        return f"verus/split/{self.output_subdir}"

    def output_dir_abs(self) -> Path:
        """Return the absolute output directory path."""
        return VERUS_SPLIT_DIR / self.output_subdir

    @classmethod
    def from_source_path(
        cls,
        source_path: str,
        module_name: Optional[str] = None,
        output_subdir: Optional[str] = None,
        file_stem: Optional[str] = None,
    ) -> "ModuleConfig":
        """
        Create a ModuleConfig from a source file path.

        Parameters:
            source_path: Path to the source file (relative to PROJECT_ROOT).
            module_name: Optional module name. If not provided, inferred from filename.
            output_subdir: Optional output subdirectory under verus/split/.
            file_stem: Optional file stem for the three-file split.

        Returns:
            ModuleConfig instance.
        """
        path = Path(source_path)

        # Infer module name from filename if not provided.
        if module_name is None:
            # e.g., "lib.rs" -> use parent dir name, "kpool.rs" -> "kpool"
            if path.stem == "lib" or path.stem == "mod":
                module_name = path.parent.name
            else:
                module_name = path.stem

        # Infer file stem from module name if not provided.
        if file_stem is None:
            file_stem = module_name

        # Infer output subdirectory if not provided.
        if output_subdir is None:
            output_subdir = module_name

        return cls(
            name=module_name,
            source_path=path,
            verify_module=module_name,
            output_subdir=output_subdir,
            file_stem=file_stem,
        )


def get_existing_modules() -> List[str]:
    """
    Get list of already verified modules from verus/split/ directory.

    Scans for directories containing .rs files (the split pattern).
    Also checks verus/ for legacy single-file modules.

    Returns:
        List of module names.
    """
    modules = []

    # Check split directory for subdirectories with .rs files.
    if VERUS_SPLIT_DIR.exists():
        for rs_file in VERUS_SPLIT_DIR.rglob("*.rs"):
            # Skip spec/proof files and mod.rs/lib.rs at the split root.
            if rs_file.name.endswith(".spec.rs") or rs_file.name.endswith(".proof.rs"):
                continue
            if rs_file.name in ("lib.rs", "mod.rs"):
                continue
            # Module name is the file stem.
            modules.append(rs_file.stem)

    # Also check legacy single-file verus/ directory.
    if VERUS_DIR.exists():
        for rs_file in VERUS_DIR.glob("*.rs"):
            if rs_file.name != "lib.rs":
                name = rs_file.stem
                if name not in modules:
                    modules.append(name)

    return sorted(set(modules))
