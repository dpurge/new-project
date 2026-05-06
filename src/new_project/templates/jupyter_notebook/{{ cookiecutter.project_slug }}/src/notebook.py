import os
import sys
from pathlib import Path

from jupyterlab.labapp import main as jupyterlab_main


def main() -> None:
    args = sys.argv[1:]
    repo_root = Path(__file__).resolve().parent.parent
    pythonpath_entries = [str(repo_root), str(repo_root / "src")]
    existing_pythonpath = os.environ.get("PYTHONPATH", "")
    existing_entries = [entry for entry in existing_pythonpath.split(os.pathsep) if entry]

    for entry in reversed(pythonpath_entries):
        if entry not in existing_entries:
            existing_entries.insert(0, entry)

    os.environ["PYTHONPATH"] = os.pathsep.join(existing_entries)

    # Force local development mode without inherited Jupyter auth prompts.
    defaults = [
        "--IdentityProvider.token=",
        "--PasswordIdentityProvider.hashed_password=",
        "--PasswordIdentityProvider.password_required=False",
    ]
    for flag in defaults:
        option = flag.split("=", 1)[0]
        if not any(arg == option or arg.startswith(f"{option}=") for arg in args):
            args.append(flag)

    sys.argv = [sys.argv[0], *args]
    jupyterlab_main()
