#!/usr/bin/env python3
"""Exercise archives on fake binaries, including link completeness and provenance."""
import json
import os
from pathlib import Path
import posixpath
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile

class ReleasePackageTests(unittest.TestCase):
    def test_archives_include_linked_docs_and_provenance(self):
        root = Path(__file__).resolve().parents[1]
        for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"]:
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                staging = Path(directory)
                for name in ["README.md", "LICENSE", "CHANGELOG.md", "SECURITY.md", "docs", "examples"]:
                    source = root / name
                    if source.is_dir(): shutil.copytree(source, staging / name)
                    else: shutil.copy2(source, staging / name)
                binary = staging / "target" / target / "release" / ("reprise.exe" if "windows" in target else "reprise")
                binary.parent.mkdir(parents=True)
                binary.write_bytes(b"fake executable for packaging only")
                env = dict(os.environ, RELEASE_TAG="v0.2.0", RELEASE_TARGET=target, GITHUB_SHA="fixture-commit")
                subprocess.run([sys.executable, str(root / "scripts/package_release.py")], cwd=staging, env=env, check=True)
                archive = next((staging / "dist").iterdir())
                if archive.suffix == ".zip":
                    with zipfile.ZipFile(archive) as package: files = {name: package.read(name) for name in package.namelist()}
                else:
                    with tarfile.open(archive) as package: files = {member.name: package.extractfile(member).read() for member in package.getmembers() if member.isfile()}
                self.assertIn("SECURITY.md", set(files))
                self.assertIn("examples/config.toml", set(files))
                self.assertIn("examples/project.toml", set(files))
                self.assertEqual(json.loads(files["build-info.json"])["commit"], "fixture-commit")
                for name, contents in files.items():
                    if name.endswith(".md"):
                        for link in contents.decode().split("](")[1:]:
                            target_link = link.split(")")[0].split("#")[0]
                            if target_link and "://" not in target_link:
                                resolved = posixpath.normpath(posixpath.join(posixpath.dirname(name), target_link))
                                self.assertIn(resolved, set(files), f"{name}: broken archive link {target_link}")

if __name__ == "__main__": unittest.main()
