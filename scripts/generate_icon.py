"""Generate native icons from the current transparent launcher emblem."""
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory

root = Path(__file__).resolve().parents[1]
source = root / "src/assets/launcher-mark.png"
cli = root / "node_modules/@tauri-apps/cli/tauri.js"
node = shutil.which("node")
if not node or not cli.is_file() or not source.is_file():
    raise SystemExit("Install Node and project dependencies; the transparent emblem must exist.")
WINDOWS_ICONS = ("32x32.png", "128x128.png", "128x128@2x.png", "icon.ico")


def generate_icons(destination: Path) -> None:
    staging = root / "artifacts"
    staging.mkdir(exist_ok=True)
    with TemporaryDirectory(prefix="tch-icons-", dir=staging) as generated:
        subprocess.run([
            node, str(cli), "icon", str(source), "--output", generated,
            "--fit", "contain",
        ], cwd=root, check=True)
        destination.mkdir(parents=True, exist_ok=True)
        for name in WINDOWS_ICONS:
            shutil.copyfile(Path(generated) / name, destination / name)


if __name__ == "__main__":
    generate_icons(root / "src-tauri/icons")
