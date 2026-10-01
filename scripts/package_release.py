#!/usr/bin/env python3
"""Package a native, smoke-tested release binary with source provenance."""
import json
import os
from pathlib import Path
import tarfile
import zipfile

release_tag = os.environ["RELEASE_TAG"]
target = os.environ["RELEASE_TARGET"]
sha = os.environ["GITHUB_SHA"]
suffix = ".exe" if "windows" in target else ""
binary = Path("target") / target / "release" / ("reprise" + suffix)
if not binary.is_file():
    raise SystemExit(f"Missing release binary: {binary}")
files = [(binary, binary.name)] + [(Path(name), name) for name in ("LICENSE", "README.md", "CHANGELOG.md", "SECURITY.md")]
files += [(file, str(file)) for file in sorted(Path("docs").glob("*.md"))]
files += [(file, str(file)) for file in sorted(Path("examples").glob("*.toml"))]
info = Path("build-info.json")
info.write_text(json.dumps({"tag": release_tag, "commit": sha, "target": target}, indent=2) + "\n")
files.append((info, info.name))
Path("dist").mkdir(exist_ok=True)
base = Path("dist") / f"reprise-{release_tag}-{target}"
if suffix:
    with zipfile.ZipFile(str(base) + ".zip", "w", zipfile.ZIP_DEFLATED) as archive:
        for file, name in files:
            archive.write(file, name)
else:
    with tarfile.open(str(base) + ".tar.gz", "w:gz") as archive:
        for file, name in files:
            archive.add(file, arcname=name)
info.unlink()
