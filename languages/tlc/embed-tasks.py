#!/usr/bin/env python3
"""Write languages/tlc/tasks.json from tlc-task.sh.

Zed joins a task's command and args with spaces and passes that string to
`login-shell -i -c`. Arguments are not quoted. Each task therefore stores the
script as one POSIX single-quoted `-c` argument and selects `/bin/sh`.

The debugger embeds that same task through the extension API. The API has no
shell field, so Zed quotes each argument again for the login shell. Fish
single quotes treat a backslash as an escape. The script is base64, and the
wrapper that decodes it contains neither a single quote nor a backslash.
`/bin/sh` decodes it into the `-c` argument of a second shell, so the TLC
program keeps the terminal's standard input.
"""

import base64
import json
import shlex
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPT = (HERE / "tlc-task.sh").read_bytes()


def shell_wrapper(script: bytes) -> str:
    payload = base64.b64encode(script).decode("ascii")
    wrapper = (
        "script=$(base64 -d <<END\n"
        f"{payload}\n"
        "END\n"
        ") || exit 1\n"
        'exec /bin/sh -c "$script" sh "$@"\n'
    )
    if "'" in wrapper or "\\" in wrapper or "$ZED_" in wrapper:
        raise SystemExit("task wrapper is not safe to quote for fish")
    return wrapper


def sh_single_quote(text: str) -> str:
    return "'" + text.replace("'", "'\\''") + "'"


def task(label: str, mode: str, tags: list[str] | None = None) -> dict:
    wrapper = shell_wrapper(SCRIPT)
    quoted = sh_single_quote(wrapper)
    if quoted != f"'{wrapper}'":
        raise SystemExit(f"{label}: wrapper needed an escaped quote")
    combined = "/bin/sh -c " + quoted + " sh " + mode
    parsed = shlex.split(combined)
    if parsed != ["/bin/sh", "-c", wrapper, "sh", mode]:
        raise SystemExit(f"{label}: quoted script does not round-trip through the shell")
    body = {
        "label": label,
        "command": "/bin/sh",
        "args": ["-c", quoted, "sh", mode],
        "cwd": "$ZED_WORKTREE_ROOT",
        # Zed still wraps the task in the login shell unless this is set.
        # /bin/sh parses the quoted script when that wrapper is fish.
        "shell": {"program": "/bin/sh"},
        "reveal": "always",
    }
    if tags:
        body["tags"] = tags
    return body


def main() -> None:
    tasks = [
        task("Run TLC program", "run", ["tlc-main"]),
        task("Check TLC program", "check"),
        task("Build TLC debug binary", "build-debug"),
    ]
    destination = HERE / "tasks.json"
    destination.write_text(json.dumps(tasks, indent=2) + "\n")


if __name__ == "__main__":
    main()
