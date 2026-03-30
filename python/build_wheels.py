"""
Build a platform-specific wheel containing the affected binary.

All wheels are published under the single "affected" package name.
pip/uv automatically selects the correct wheel by platform tag.

Usage: python build_wheels.py <target> <binary_path> <version> <output_dir>
Example: python build_wheels.py x86_64-unknown-linux-gnu ./affected 0.2.1 dist/
"""
import argparse
import hashlib
import os
import shutil
import stat
import tempfile
import zipfile

# Map Rust target triples to Python wheel platform tags
PLATFORM_TAGS = {
    "x86_64-unknown-linux-gnu": "manylinux_2_17_x86_64.manylinux2014_x86_64",
    "aarch64-unknown-linux-gnu": "manylinux_2_17_aarch64.manylinux2014_aarch64",
    "x86_64-apple-darwin": "macosx_10_12_x86_64",
    "aarch64-apple-darwin": "macosx_11_0_arm64",
    "x86_64-pc-windows-msvc": "win_amd64",
}

# Python source files to bundle in every wheel
PYTHON_SOURCES = {
    "affected/__init__.py": '"""affected: Detect affected packages. Run only what matters."""\n',
    "affected/__main__.py": (
        '"""Allow running as `python -m affected`."""\n'
        "from affected._main import main\n\nmain()\n"
    ),
    "affected/_main.py": None,  # read from disk
}


def build_wheel(target, binary_path, version, output_dir):
    tag = PLATFORM_TAGS[target]
    wheel_name = f"affected-{version}-py3-none-{tag}.whl"

    # Read the real _main.py source
    main_src = os.path.join(
        os.path.dirname(__file__),
        "affected", "src", "affected", "_main.py",
    )
    with open(main_src) as f:
        main_py_content = f.read()

    with tempfile.TemporaryDirectory() as tmpdir:
        # Create package directory
        pkg_dir = os.path.join(tmpdir, "affected")
        bin_dir = os.path.join(pkg_dir, "bin")
        os.makedirs(bin_dir)

        # Write Python source files
        for relpath, content in PYTHON_SOURCES.items():
            fpath = os.path.join(tmpdir, relpath)
            with open(fpath, "w") as f:
                f.write(main_py_content if content is None else content)

        # Copy binary
        binary_dest_name = "affected.exe" if "windows" in target else "affected"
        dest = os.path.join(bin_dir, binary_dest_name)
        shutil.copy2(binary_path, dest)
        os.chmod(dest, os.stat(dest).st_mode | stat.S_IEXEC)

        # dist-info directory
        dist_info = os.path.join(tmpdir, f"affected-{version}.dist-info")
        os.makedirs(dist_info)

        with open(os.path.join(dist_info, "METADATA"), "w") as f:
            f.write("Metadata-Version: 2.1\n")
            f.write(f"Name: affected\n")
            f.write(f"Version: {version}\n")
            f.write("Summary: Detect affected packages. Run only what matters.\n")
            f.write("License: MIT\n")
            f.write("Requires-Python: >=3.8\n")

        with open(os.path.join(dist_info, "WHEEL"), "w") as f:
            f.write("Wheel-Version: 1.0\n")
            f.write("Generator: affected-build-wheels\n")
            f.write("Root-Is-Purelib: false\n")
            f.write(f"Tag: py3-none-{tag}\n")

        # entry_points.txt for the console script
        with open(os.path.join(dist_info, "entry_points.txt"), "w") as f:
            f.write("[console_scripts]\n")
            f.write("affected = affected._main:main\n")

        # RECORD (must list all files with hashes)
        record_path = os.path.join(dist_info, "RECORD")
        records = []
        for root, _dirs, files in os.walk(tmpdir):
            for fname in files:
                fpath = os.path.join(root, fname)
                relpath = os.path.relpath(fpath, tmpdir)
                if relpath.endswith("RECORD"):
                    continue
                with open(fpath, "rb") as fobj:
                    digest = hashlib.sha256(fobj.read()).hexdigest()
                    size = os.path.getsize(fpath)
                records.append(f"{relpath},sha256={digest},{size}")
        records.append(f"{os.path.relpath(record_path, tmpdir)},,")

        with open(record_path, "w") as f:
            f.write("\n".join(records) + "\n")

        # Build the wheel (just a zip file)
        os.makedirs(output_dir, exist_ok=True)
        wheel_path = os.path.join(output_dir, wheel_name)
        with zipfile.ZipFile(wheel_path, "w", zipfile.ZIP_DEFLATED) as whl:
            for root, _dirs, files in os.walk(tmpdir):
                for fname in files:
                    fpath = os.path.join(root, fname)
                    arcname = os.path.relpath(fpath, tmpdir)
                    whl.write(fpath, arcname)

        print(f"Built: {wheel_path}")
        return wheel_path


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Build a platform-specific wheel for affected."
    )
    parser.add_argument("target", choices=list(PLATFORM_TAGS.keys()))
    parser.add_argument("binary_path")
    parser.add_argument("version")
    parser.add_argument("output_dir")
    args = parser.parse_args()
    build_wheel(args.target, args.binary_path, args.version, args.output_dir)
