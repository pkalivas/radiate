"""Execute every user-guide Python snippet to keep the docs runnable.

Each file under ``docs/source/src/python`` is a self-contained, runnable program.
The rendered user guide shows only the regions between ``# --8<-- [start:NAME]`` and
``# --8<-- [end:NAME]`` markers (via ``pymdownx.snippets``), but the *whole* file is
executed here. If a snippet drifts from the real API, this test fails — that is the
point. See the doc-testing plan for the full workflow.
"""

from __future__ import annotations

import runpy
from pathlib import Path

import pytest

import radiate as rd

# Plotting snippets must render head-lessly (no GUI window) during tests.
try:
    import matplotlib

    matplotlib.use("Agg")
except ImportError:
    pass

# docs/source/src/python  (this file lives at py-radiate/tests/docs/test_user_guide.py)
SNIPPET_ROOT = (
    Path(__file__).resolve().parents[3] / "docs" / "source" / "src" / "python"
)

# Path segments whose snippets are not executed here: they write files, need a TTY,
# or are intentionally long-running. Matched against the path relative to SNIPPET_ROOT.
EXCLUDE_SEGMENTS = {"ui", "checkpoint"}


def _excluded(rel: Path) -> bool:
    # Skip excluded directory segments, and end-to-end "*_showcase.py" files
    # (long-running, plotting demos) — same spirit as polars excluding its
    # visualization snippets from the run test. Their API is still validated
    # elsewhere and their `--8<--` includes are checked by `mkdocs build --strict`.
    if EXCLUDE_SEGMENTS & set(rel.parts):
        return True
    return rel.stem.endswith("_showcase")


def _discover() -> list[Path]:
    if not SNIPPET_ROOT.exists():
        return []
    return sorted(
        p
        for p in SNIPPET_ROOT.rglob("*.py")
        if not _excluded(p.relative_to(SNIPPET_ROOT))
    )


SNIPPETS = _discover()


@pytest.mark.parametrize(
    "path",
    SNIPPETS,
    ids=[str(p.relative_to(SNIPPET_ROOT)) for p in SNIPPETS],
)
def test_doc_snippet_runs(path: Path) -> None:
    """A user-guide snippet file must execute top-to-bottom without raising."""
    rd.random.seed(0)  # determinism for any randomized construction
    runpy.run_path(str(path), run_name="__main__")
