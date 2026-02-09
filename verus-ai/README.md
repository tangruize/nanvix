# Verus AI Verification Workflow

Automated formal verification of Rust code using AI agents and Verus.

## Overview

This tool implements the **Prover-Reviewer Iteration** methodology:

1. **Prover** (Claude Opus 4.5): Generates Verus specifications and proofs
2. **Reviewers** (3 models): Critique the verification for completeness
3. **Iteration**: Prover addresses issues until all reviewers assign A+

## Quick Start

```bash
# Show demo and usage
./run.py demo

# Verify a new module (specify source file path)
./run.py verify src/kernel/src/mm/phys/upool.rs

# Verify with custom module name
./run.py verify src/libs/bitmap/src/lib.rs --name bitmap

# Resume an interrupted verification
./run.py verify upool --resume

# Check workflow status
./run.py status

# Check for cheating patterns
./run.py check bitmap

# Generate summary report
./run.py report
```

## Architecture

```
verus-ai/
├── __init__.py       # Package init
├── config.py         # Configuration (models, paths, timeouts)
├── prompts.py        # Prompt templates for prover/reviewers
├── copilot.py        # Copilot CLI interface with logging
├── guardrails.py     # Cheating detection, verification, git
├── workflow.py       # Main orchestration logic
├── report.py         # Report generation
├── utils.py          # Utility functions
├── run.py            # Quick runner script
└── scripts/
    └── verify.sh     # Verus verification with auto-commit

verus-ai-history/     # All logs and reviews (auto-committed)
├── logs/             # Verification logs, copilot session logs
└── reviews/          # Reviewer output files
```

## Workflow Details

### Phase 1: Initial Prover

The prover generates an initial Verus verification:
- Creates View types for abstract state
- Defines invariants for all operations
- Adds requires/ensures contracts
- Iterates until `verus` passes

### Phase 2: Multi-Model Review

Three reviewers independently analyze the code:
- **claude-opus-4.6**: Comprehensive primary review
- **gpt-5.2-codex**: Alternative perspective
- **gemini-3-pro-preview**: Additional coverage

Each reviewer checks:
- Coverage: All functions verified
- Specifications: Adequate pre/post conditions
- Soundness: No unjustified assumes
- Equivalence: Matches original semantics

### Phase 3: Iteration

The prover addresses reviewer issues:
- Fixes valid issues
- Justifies rejected issues
- Continues until all A+ grades

### Guardrails

Automated checks prevent cheating:
- `assume` statements: Must be zero
- `external_body`: Must be justified
- Verification timeout: 60s per module
- Git commits: Track all changes

## Configuration

Edit `config.py` to customize:

```python
# Models
PROVER_MODEL = "claude-opus-4.6"
REVIEWER_MODELS = [
    "claude-opus-4.6",
    "gpt-5.2-codex", 
    "gemini-3-pro-preview",
]

# Limits
MAX_REVIEW_ITERATIONS = 5
GRADE_THRESHOLD = "A"
```

## Adding New Modules

Simply run verify with the source file path:

```bash
./run.py verify src/path/to/new_module.rs
```

The module name is automatically inferred from the filename. Use `--name` to override.

## Output Files

- `verus/<module>.rs`: Verified Verus code
- `histories/reviewers/<module>_<model>_iter<N>.md`: Review files
- `verus-ai/logs/workflow_state_<module>.json`: Workflow state
- `verus-ai/logs/*.log`: Detailed interaction logs

## Trust Boundary

The verification maintains a clear trust decomposition:

| Level | Component | Trust |
|-------|-----------|-------|
| 0 | Proofs (327 properties) | Mechanically verified by Verus |
| 1 | Specifications (~200 lines) | Human-reviewed |
| 2 | external_body (10 functions) | Explicitly audited |

## Requirements

- Python 3.10+
- GitHub Copilot CLI (`copilot` command)
- Verus (`verus` command)
- Git

## License

Copyright(c) The Maintainers of Nanvix.
Licensed under the MIT License.
