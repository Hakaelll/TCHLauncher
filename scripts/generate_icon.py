"""Generate native icons from the current transparent launcher emblem."""
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
source = root / "src/assets/launcher-emblem-cutout.png"
cli = root / "node_modules/@tauri-apps/cli/tauri.js"
node = shutil.which("node")
if not node or not cli.is_file() or not source.is_file():
    raise SystemExit("Install Node and project dependencies; the transparent emblem must exist.")
subprocess.run([
    node, str(cli), "icon", str(source), "--output", str(root / "src-tauri/icons"),
    "--fit", "contain",
], cwd=root, check=True)
