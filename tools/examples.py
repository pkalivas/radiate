#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[1]
EXAMPLES_DIR = ROOT / "examples"
MANIFEST_PATH = EXAMPLES_DIR / "examples.yaml"


@dataclass
class Example:
    id: str
    manifest: dict[str, str]

    timeout: int = 30
    tags: list[str] = field(default_factory=list)
    skip_on_ci: bool = False

    def command(self, language: str) -> list[str]:
        if language == "python":
            py_path = self.manifest.get("py")
            if not py_path:
                raise ValueError(f"{self.id}: no Python manifest entry")
            return [
                "uv",
                "run",
                py_path,
            ]

        if language == "rust":
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
    defaults = data.get("defaults", {})
    rust_defaults = defaults.get("rust", {})

    examples: list[Example] = []
    for entry in data.get("examples", []):
        manifest = dict(entry.get("manifest", {}))
        timeout = int(entry.get("timeout", rust_defaults.get("timeout", 30)))
        tags = list(entry.get("tags", []))
        skip_on_ci = bool(entry.get("skip_on_ci", False))

        examples.append(
            Example(
                id=entry["id"],
                manifest=manifest,
                timeout=timeout,
                tags=tags,
                skip_on_ci=skip_on_ci,
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
    tag: str | None = None,
    ci_only: bool = False,
) -> list[tuple[Example, str]]:
    selected: list[tuple[Example, str]] = []

    for ex in examples:
        if ex_id and ex.id != ex_id:
            continue
        if tag and tag not in ex.tags:
            continue
        if ci_only and ex.skip_on_ci:
            continue

        available_languages: list[str] = []
        if "py" in ex.manifest:
            available_languages.append("python")
        if "rs" in ex.manifest:
            available_languages.append("rust")

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
            timeout=example.timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        print(f"FAILED: {example.id} [{language}] timed out after {example.timeout}s")
        return False
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
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("list")

    run_parser = sub.add_parser("run")
    run_parser.add_argument("--id")
    run_parser.add_argument("--language", choices=["python", "rust"])
    run_parser.add_argument("--tag")
    run_parser.add_argument("--ci", action="store_true")

    args = parser.parse_args()
    examples = load_manifest()

    if args.command == "list":
        list_examples(examples)
        return 0

    if args.command == "run":
        selected = select_examples(
            examples,
            ex_id=args.id,
            language=args.language,
            tag=args.tag,
            ci_only=args.ci,
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
