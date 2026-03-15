#!/usr/bin/env python3
"""
Simple tool to manage and run examples for Radiate. Examples are defined in a
manifest file (examples/examples.yaml) and can be run with the `just example` command.

Ex: `just example --id nn --lang py` will run the Python version of the "nn" example, while
"""

from __future__ import annotations

import argparse
import subprocess
import yaml  # type: ignore

from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
EXAMPLES_DIR = ROOT / "examples"
MANIFEST_PATH = EXAMPLES_DIR / "examples.yaml"


@dataclass
class Example:
    id: str
    manifest: dict[str, str]

    def command(self, language: str) -> list[str]:
        if language == "py":
            py_path = self.manifest.get("py")
            if not py_path:
                raise ValueError(f"{self.id}: no Python manifest entry")
            return [
                "uv",
                "run",
                py_path,
            ]

        if language == "rs":
            rs_manifest = self.manifest.get("rs")
            if not rs_manifest:
                raise ValueError(f"{self.id}: no Rust manifest entry")
            return [
                "cargo",
                "run",
                "--manifest-path",
                rs_manifest,
            ]

        raise ValueError(f"{self.id}: unsupported language {language}")


def load_manifest() -> list[Example]:
    if not MANIFEST_PATH.exists():
        return []

    data = yaml.safe_load(MANIFEST_PATH.read_text(encoding="utf-8")) or {}

    examples: list[Example] = []
    for entry in data.get("examples", []):
        manifest = dict(entry.get("manifest", {}))

        examples.append(
            Example(
                id=entry["id"],
                manifest=manifest,
            )
        )

    return examples


def list_examples(examples: list[Example]):
    print("Examples:")
    python_examples = [ex for ex in examples if "py" in ex.manifest]
    rust_examples = [ex for ex in examples if "rs" in ex.manifest]
    if not python_examples and not rust_examples:
        print("  No examples found.")
        return 0

    for ex in python_examples:
        print(f"{ex.id:28} {'python':12}")

    print()

    for ex in rust_examples:
        print(f"{ex.id:28} {'rust':12} ")


def select_examples(
    examples: list[Example],
    *,
    ex_id: str | None = None,
    language: str | None = None,
) -> list[tuple[Example, str]]:
    selected: list[tuple[Example, str]] = []

    for ex in examples:
        if ex_id and ex.id != ex_id:
            continue

        available_languages: list[str] = []
        if "py" in ex.manifest:
            available_languages.append("py")
        if "rs" in ex.manifest:
            available_languages.append("rs")

        if language:
            if language in available_languages:
                selected.append((ex, language))
            continue

        for lang in available_languages:
            selected.append((ex, lang))

    return selected


def run_example(example: Example, language: str) -> bool:
    cmd = example.command(language)

    print(f"\n==> Running {example.id} [{language}]")
    print("+", " ".join(cmd))

    try:
        result = subprocess.run(
            cmd,
            cwd=ROOT,
            check=False,
        )
    except FileNotFoundError as e:
        print(f"FAILED: {example.id} [{language}] could not start: {e}")
        return False

    if result.returncode != 0:
        print(f"FAILED: {example.id} [{language}] exited with code {result.returncode}")
        return False

    print(f"PASSED: {example.id} [{language}]")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=False)

    sub.add_parser("list")

    run_parser = sub.add_parser("run")
    run_parser.add_argument("--id")
    run_parser.add_argument("--lang", choices=["py", "rs"])

    args = parser.parse_args()
    examples = load_manifest()

    if args.command == "list":
        list_examples(examples)
        return 0

    if args.command == "run":
        selected = select_examples(
            examples,
            ex_id=args.id,
            language=args.lang,
        )

        if not selected:
            print("No examples matched.")
            return 1

        failures = sum(not run_example(ex, lang) for ex, lang in selected)
        print(f"\nSummary: {len(selected) - failures} passed, {failures} failed")
        return 1 if failures else 0

    return 1


if __name__ == "__main__":
    raise SystemExit(main())
