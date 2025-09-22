import sys
import subprocess
from pathlib import Path


def main() -> int:
    repo_root = Path(__file__).resolve().parents[1]
    tong_py = repo_root / "tong.py"

    # Collect all .tong files under examples/ (including nested folders)
    examples_dir = repo_root / "examples"
    files = sorted(examples_dir.rglob("*.tong"))

    if not files:
        print("No .tong examples found.")
        return 1

    print(f"Found {len(files)} example(s). Running...\n")

    passed = []
    failed = []

    for f in files:
        rel = f.relative_to(repo_root)
        print(f"==> {rel}")
        result = subprocess.run([sys.executable, str(tong_py), str(f)], capture_output=True, text=True, check=False)
        if result.returncode == 0:
            print("PASS\n")
            passed.append(rel)
        else:
            print("FAIL\n")
            print(result.stdout)
            print(result.stderr)
            failed.append(rel)

    print("Summary:")
    print(f"  Passed: {len(passed)}")
    print(f"  Failed: {len(failed)}")
    if failed:
        print("\nFailures:")
        for rel in failed:
            print(f"  - {rel}")

    return 0 if not failed else 2


if __name__ == "__main__":
    sys.exit(main())
