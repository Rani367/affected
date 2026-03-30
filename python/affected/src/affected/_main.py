"""Entry point that finds and execs the platform-specific binary."""
import os
import sys


def _find_binary():
    """Find the affected binary bundled in this package."""
    pkg_dir = os.path.dirname(os.path.abspath(__file__))
    binary_name = "affected.exe" if sys.platform == "win32" else "affected"
    binary_path = os.path.join(pkg_dir, "bin", binary_name)
    if os.path.isfile(binary_path):
        return binary_path

    print("error: could not find the 'affected' binary.", file=sys.stderr)
    print("hint: try reinstalling with: pip install --force-reinstall affected", file=sys.stderr)
    sys.exit(1)


def main():
    binary = _find_binary()
    args = [binary] + sys.argv[1:]

    if sys.platform == "win32":
        import subprocess

        result = subprocess.run(args)
        sys.exit(result.returncode)
    else:
        os.execvp(binary, args)
