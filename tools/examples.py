#!/usr/bin/env python3
"""
Simple tool to manage and run examples
"""

import subprocess
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLES_DIR = ROOT / "examples"
PY_EXAMPLES_DIR = EXAMPLES_DIR / "python"
RS_EXAMPLES_DIR = EXAMPLES_DIR / "rust"


@dataclass
class Example:
    name: str
    path: Path
    lang: str


def load_py_examples() -> list[Example]:
    return list(
        map(
            lambda py_file: Example(
                name=py_file.stem,
                path=py_file,
                lang="python",
            ),
            filter(
                lambda py_file: py_file.is_file() and not py_file.name.startswith("_"),
                PY_EXAMPLES_DIR.glob("*.py"),
            ),
        )
    )


def load_rs_examples() -> list[Example]:
    return list(
        map(
            lambda rs_dir: Example(
                name=rs_dir.name,
                path=rs_dir,
                lang="rust",
            ),
            filter(
                lambda rs_dir: rs_dir.relative_to(RS_EXAMPLES_DIR).as_posix()
                and (rs_dir / "Cargo.toml").is_file(),
                map(
                    lambda cargo_toml: cargo_toml.parent,
                    sorted(
                        RS_EXAMPLES_DIR.rglob("Cargo.toml"),
                    ),
                ),
            ),
        )
    )


def run_example(example: Example, extra_args: list[str]) -> int:
    print(f"\n==> Running {example.name} [{example.lang}]\n")

    if example.lang == "python":
        cmd = ["uv", "run", "python", str(example.path), *extra_args]
        cwd = ROOT
    elif example.lang == "rust":
        cmd = [
            "cargo",
            "run",
            "--manifest-path",
            str(example.path / "Cargo.toml"),
            "--release",
            "--",
            *extra_args,
        ]
        cwd = ROOT

    else:
        raise ValueError(f"unsupported example language: {example.lang}")

    print("$ " + " ".join(cmd))

    result = subprocess.run(cmd, cwd=cwd)

    return result.returncode


def main() -> int:
    py_examples = load_py_examples()
    rs_examples = load_rs_examples()

    print("language:")
    print(" [1] Python")
    print(" [2] Rust")
    lang_choice = input(":: ")

    if lang_choice == "1":
        examples = py_examples
    elif lang_choice == "2":
        examples = rs_examples
    else:
        print("Invalid choice")
        return 1

    print("\nAvailable examples:")
    for i, example in enumerate(examples):
        print(f" [{i + 1}] {example.name}")

    try:
        example_choice = input(":: ")
        example_to_run = examples[int(example_choice) - 1]

        run_example(example_to_run, extra_args=[])
    except Exception as e:
        print(f"Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
