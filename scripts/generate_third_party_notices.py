"""Collect installed dependency metadata and upstream notices, offline.

Run from the repository with Python 3 after installing locked Rust/pnpm dependencies.
This inventories the Windows Cargo resolve graph (including build dependencies),
the frontend runtime packages and Monocraft. It is not a legal compatibility audit.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "docs" / "legal"


def license_files(folder):
    candidates = []
    for path in sorted(folder.iterdir()):
        if path.is_file() and re.match(r"(?i)^(license|licence|copying|notice|copyright)([._-]|$)", path.name):
            candidates.append(path)
        elif path.is_dir() and path.name.lower() in {"license", "licenses", "licences"}:
            candidates.extend(p for p in sorted(path.rglob("*")) if p.is_file())
    # Native libraries may carry their own licenses below a sys crate.
    for subfolder in ("liblzma", "bzip2", "sqlite3", "aws-lc", "vendor"):
        sub = folder / subfolder
        if sub.is_dir():
            candidates.extend(p for p in sorted(sub.iterdir()) if p.is_file() and re.match(r"(?i)^(license|licence|copying|notice|copyright)([._-]|$)", p.name))
    return [p for p in candidates if p.stat().st_size <= 512_000]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--manifest-path", str(ROOT / "src-tauri/Cargo.toml"),
        "--offline", "--locked", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc",
    ], cwd=ROOT, text=True, encoding="utf-8"))
    included = {node["id"] for node in metadata["resolve"]["nodes"]}
    packages = []
    for item in metadata["packages"]:
        if item["id"] not in included or not item.get("source"):
            continue
        packages.append(("cargo", item["name"], item["version"], item.get("license"), item.get("repository") or item.get("homepage"), Path(item["manifest_path"]).parent))
    # These are the frontend runtime dependency graph in the installed lockfile.
    for name in ("react", "react-dom", "scheduler", "@tauri-apps/api"):
        folder = ROOT / "node_modules" / name
        if not folder.exists() and name == "scheduler":
            folders = list((ROOT / "node_modules/.pnpm").glob("scheduler@*/node_modules/scheduler"))
            if len(folders) != 1:
                raise RuntimeError("Resolve exactly one installed scheduler version before generating notices")
            folder = folders[0]
        item = json.loads((folder / "package.json").read_text(encoding="utf-8"))
        repository = item.get("repository")
        if isinstance(repository, dict):
            repository = repository.get("url")
        packages.append(("npm", name, item["version"], item.get("license"), repository, folder))
    records, sections, missing = [], [], []
    for ecosystem, name, version, license_id, source, folder in sorted(packages):
        files = license_files(folder)
        local_upstream = OUT / "upstream" / f"{name}-{version}"
        if not files and local_upstream.is_dir():
            files = license_files(local_upstream)
        notices = []
        for path in files:
            raw = path.read_bytes()
            relative = path.relative_to(local_upstream if path.is_relative_to(local_upstream) else folder).as_posix()
            notices.append({"file": relative, "sha256": hashlib.sha256(raw).hexdigest()})
            sections.append(f"\n{'=' * 72}\n{ecosystem}: {name} {version}\nDeclared license: {license_id or 'not declared'}\nUpstream file: {relative}\nSource: {source or 'not declared'}\n{'=' * 72}\n\n" + raw.decode("utf-8", errors="replace"))
        if not files:
            missing.append(f"{ecosystem}:{name}@{version}")
        archive = f"https://crates.io/api/v1/crates/{name}/{version}/download" if ecosystem == "cargo" else f"https://www.npmjs.com/package/{name}/v/{version}"
        records.append({"ecosystem": ecosystem, "name": name, "version": version, "license": license_id, "source": source, "sourceArchive": archive, "notices": notices})
        if name == "libsqlite3-sys":
            # The binding's MIT license does not describe the bundled SQLite C core.
            amalgamation = (folder / "sqlite3/sqlite3.c").read_text(encoding="utf-8")
            position = amalgamation.index("The author disclaims copyright")
            start = amalgamation.rfind("/*", 0, position)
            end = amalgamation.index("*/", position) + 2
            notice = amalgamation[start:end]
            version_match = re.search(r'#define SQLITE_VERSION\s+"([^"]+)"', amalgamation)
            if not version_match:
                raise RuntimeError("Cannot identify the bundled SQLite version")
            sections.append("\n" + "=" * 72 + "\nSQLite " + version_match[1] + " — bundled C core dedication\nSource: sqlite3/sqlite3.c in " + archive + "\n" + "=" * 72 + "\n\n" + notice + "\n")
            records.append({"ecosystem": "native", "name": "SQLite", "version": version_match[1], "license": "blessing", "source": "https://www.sqlite.org/", "sourceArchive": archive, "notices": [{"file": "sqlite3/sqlite3.c (dedication comment)", "sha256": hashlib.sha256(notice.encode("utf-8")).hexdigest()}]})
    font = ROOT / "public/fonts/OFL-Monocraft.txt"
    sections.append("\n" + "=" * 72 + "\nMonocraft 4.2.1 — SIL Open Font License 1.1\n" + "=" * 72 + "\n\n" + font.read_text(encoding="utf-8"))
    records.append({"ecosystem": "asset", "name": "Monocraft", "version": "4.2.1", "license": "OFL-1.1", "source": "https://github.com/IdreesInc/Monocraft", "notices": [{"file": "OFL-Monocraft.txt", "sha256": hashlib.sha256(font.read_bytes()).hexdigest()}]})
    nsis = OUT / "upstream/NSIS-COPYING.txt"
    if nsis.exists():
        sections.append("\n" + "=" * 72 + "\nNSIS Windows installer — upstream COPYING\n" + "=" * 72 + "\n\n" + nsis.read_text(encoding="utf-8"))
        records.append({"ecosystem": "installer", "name": "NSIS", "source": "https://nsis.sourceforge.io/", "notices": [{"file": "NSIS-COPYING.txt", "sha256": hashlib.sha256(nsis.read_bytes()).hexdigest()}]})
    header = "TCHLauncher — third-party license texts\n\nCollected from the installed, locked dependencies for the Windows build.\nIncludes Cargo build dependencies and transitive packages; inclusion in this\ninventory does not mean every package is embedded in the final executable.\nEach dependency retains its own license and original copyright notices.\nMonocraft is distributed unmodified. See THIRD_PARTY_NOTICES.md for asset\nprovenance, downloaded software and the inventory's verification limits.\n"
    (OUT / "THIRD_PARTY_LICENSES.txt").write_text(header + "".join(sections), encoding="utf-8")
    (OUT / "dependency-licenses.json").write_text(json.dumps({"format": 1, "target": "x86_64-pc-windows-msvc", "packages": records, "missingLocalLicenseTexts": missing}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Recorded {len(records)} packages, {len(sections)} notice files; missing local texts: {len(missing)}")
    if missing:
        print("\n".join(missing))


if __name__ == "__main__":
    main()
