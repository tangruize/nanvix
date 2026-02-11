# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Copilot agent interface for the Verus AI verification workflow.
"""

import subprocess
import json
import re
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional

from config import PROJECT_ROOT, LOGS_DIR, PROVER_TIMEOUT, REVIEWER_TIMEOUT


@dataclass
class CopilotSession:
    """Represents a Copilot chat session."""

    session_id: Optional[str] = None
    model: str = "claude-opus-4.6"
    log_file: Optional[Path] = None


def _write_log(
    log_file: Path,
    model: str,
    prompt: str,
    session_id: Optional[str],
    output: str,
    exit_code: int,
    duration: float,
) -> None:
    """Write a detailed log file for a copilot session."""
    with open(log_file, "w") as f:
        f.write("=== Copilot Session ===\n")
        f.write(f"Timestamp: {datetime.now().isoformat()}\n")
        f.write(f"Model: {model}\n")
        f.write(f"Session: {session_id or 'new'}\n")
        f.write(f"Duration: {duration:.1f}s\n")
        f.write(f"Exit code: {exit_code}\n")
        f.write("\n=== Prompt ===\n")
        f.write(prompt)
        f.write("\n\n=== Output ===\n")
        f.write(output)
        f.write("\n=== End ===\n")


def run_copilot(
    prompt: str,
    model: str,
    session: Optional[CopilotSession] = None,
    timeout: int = 600,
    log_prefix: str = "copilot",
    module_name: Optional[str] = None,
) -> tuple[str, CopilotSession]:
    """
    Run a Copilot command and return the output.

    Parameters:
        prompt: The prompt to send to Copilot.
        model: The model to use.
        session: Optional session to resume.
        timeout: Timeout in seconds.
        log_prefix: Prefix for log filename.
        module_name: Optional module name for organizing logs into subdirectory.

    Returns:
        Tuple of (output, session) where session contains the session ID.
    """
    cmd = ["copilot", "--allow-all-tools", "--allow-all-paths", "--model", model]

    # Always include the prompt.
    cmd.extend(["-p", prompt])

    # Add resume flag if we have a session.
    if session and session.session_id:
        cmd.extend(["--resume", session.session_id])

    # Create log directory (with module subdirectory if specified).
    if module_name:
        log_dir = LOGS_DIR / module_name
    else:
        log_dir = LOGS_DIR
    log_dir.mkdir(parents=True, exist_ok=True)

    # Generate log filename.
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    model_short = model.split("-")[0]
    log_file = log_dir / f"{log_prefix}_{model_short}_{timestamp}.txt"

    # Write header to log file.
    start_time = datetime.now()
    with open(log_file, "w") as f:
        f.write(f"=== Copilot Session ===\n")
        f.write(f"Timestamp: {start_time.isoformat()}\n")
        f.write(f"Model: {model}\n")
        f.write(f"Session: {session.session_id if session else 'new'}\n")
        f.write(f"Command: {' '.join(cmd[:5])}...\n")
        f.write(f"\n=== Prompt ===\n")
        f.write(prompt)  # Write full prompt for reproducibility.
        f.write(f"\n\n=== Output (streaming) ===\n")
        f.flush()

    # Run the command with real-time output streaming to file.
    exit_code = 0
    output_lines = []
    try:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            cwd=PROJECT_ROOT,
            bufsize=1,  # Line buffered.
        )

        # Stream output to file in real-time.
        with open(log_file, "a") as f:
            for line in process.stdout:
                f.write(line)
                f.flush()
                output_lines.append(line)

        process.wait(timeout=timeout)
        exit_code = process.returncode
        output = "".join(output_lines)

    except subprocess.TimeoutExpired:
        process.kill()
        output = "".join(output_lines) + f"\nERROR: Command timed out after {timeout} seconds"
        exit_code = 124
    except Exception as e:
        output = "".join(output_lines) + f"\nERROR: {e}"
        exit_code = 1

    duration = (datetime.now() - start_time).total_seconds()

    # Write footer to log file.
    with open(log_file, "a") as f:
        f.write(f"\n\n=== Session End ===\n")
        f.write(f"Duration: {duration:.1f}s\n")
        f.write(f"Exit code: {exit_code}\n")

    # Try to extract session ID from copilot session-state or output.
    new_session = CopilotSession(model=model, log_file=log_file)

    # Method 1: If resuming an existing session, keep the same session ID.
    if session and session.session_id:
        new_session.session_id = session.session_id

    # Method 2: Get the most recently modified session from ~/.copilot/session-state/.
    # This is the most reliable method since copilot doesn't print session ID in output.
    # Newer copilot versions store sessions as directories with events.jsonl inside;
    # older versions store them as plain .jsonl files.
    if not new_session.session_id:
        session_state_dir = Path.home() / ".copilot" / "session-state"
        if session_state_dir.exists():
            try:
                # Collect candidates: directories with events.jsonl + plain .jsonl files.
                candidates: list[tuple[float, str]] = []

                # New-style: directories containing events.jsonl.
                for d in session_state_dir.iterdir():
                    if d.is_dir():
                        events_file = d / "events.jsonl"
                        if events_file.exists():
                            candidates.append((events_file.stat().st_mtime, d.name))

                # Old-style: plain .jsonl files.
                for f in session_state_dir.glob("*.jsonl"):
                    candidates.append((f.stat().st_mtime, f.stem))

                if candidates:
                    # Pick the most recently modified.
                    candidates.sort(key=lambda x: x[0], reverse=True)
                    new_session.session_id = candidates[0][1]
            except Exception:
                pass

    # Method 3 (fallback): Look for session ID in output (unlikely to work).
    if not new_session.session_id:
        session_patterns = [
            r"--resume[=\s]+([a-fA-F0-9-]{36})",  # --resume=UUID or --resume UUID
            r"[Ss]ession[:\s]+([a-fA-F0-9-]{36})",  # Session: UUID
        ]
        for pattern in session_patterns:
            session_match = re.search(pattern, output)
            if session_match:
                new_session.session_id = session_match.group(1)
                break

    # Write log file.
    _write_log(
        log_file,
        model,
        prompt,
        new_session.session_id,
        output,
        exit_code,
        duration,
    )

    return output, new_session


def run_prover(
    prompt: str,
    model: str,
    timeout: int = PROVER_TIMEOUT,
    module_name: Optional[str] = None,
) -> tuple[str, CopilotSession]:
    """
    Run the prover agent.

    Parameters:
        prompt: The prover prompt.
        model: The model to use.
        timeout: Timeout in seconds (default from config).
        module_name: Optional module name for organizing logs.

    Returns:
        Tuple of (output, session).
    """
    return run_copilot(prompt, model, timeout=timeout, log_prefix="prover", module_name=module_name)


def run_reviewer(
    prompt: str,
    model: str,
    session: Optional[CopilotSession] = None,
    timeout: int = REVIEWER_TIMEOUT,
    module_name: Optional[str] = None,
) -> tuple[str, CopilotSession]:
    """
    Run a reviewer agent.

    Parameters:
        prompt: The reviewer prompt.
        model: The model to use.
        session: Optional session to resume for follow-up reviews.
        timeout: Timeout in seconds (default from config).
        module_name: Optional module name for organizing logs.

    Returns:
        Tuple of (output, session).
    """
    return run_copilot(prompt, model, session=session, timeout=timeout, log_prefix="reviewer", module_name=module_name)


def save_session(session: CopilotSession, name: str) -> Path:
    """
    Save session info to a JSON file.

    Parameters:
        session: The session to save.
        name: Name for the session file.

    Returns:
        Path to the saved file.
    """
    LOGS_DIR.mkdir(parents=True, exist_ok=True)
    session_file = LOGS_DIR / f"session_{name}.json"

    data = {
        "session_id": session.session_id,
        "model": session.model,
    }

    with open(session_file, "w") as f:
        json.dump(data, f, indent=2)

    return session_file


def load_session(name: str) -> Optional[CopilotSession]:
    """
    Load session info from a JSON file.

    Parameters:
        name: Name of the session file.

    Returns:
        CopilotSession or None if not found.
    """
    session_file = LOGS_DIR / f"session_{name}.json"

    if not session_file.exists():
        return None

    with open(session_file, "r") as f:
        data = json.load(f)

    return CopilotSession(
        session_id=data.get("session_id"),
        model=data.get("model", "claude-opus-4.6"),
    )
