"""Execute every user-guide Python snippet to keep the docs runnable.

MIRROR CONVENTION
-----------------
Each user-guide page that contains Python examples has a 1:1 mirror host file at the
same relative path:

    docs/source/<path>.md   <->   docs/source/src/python/<path>.py

So to edit a page's Python snippets, open the file at the matching path. The page
includes regions of that file via ``--8<-- "python/<path>.py:NAME"`` (pymdownx.snippets);
the file marks them with ``# --8<-- [start:NAME]`` / ``# --8<-- [end:NAME]``. Setup that
shouldn't appear in the docs (fitness fns, sample data) goes *outside* the markers. The
*whole* file is executed by this test, so if a snippet drifts from the real API it fails.

EXCEPTIONS to the 1:1 mirror:
  * ``<path>_showcase.py`` — long-running / UI / plotting / file-IO demos. EXCLUDED from
    this run-test (see ``_excluded``); still rendered and include-validated by
    ``mkdocs build --strict``. A page may have both ``<path>.py`` and ``<path>_showcase.py``.
  * ``misc/randomness.py`` mirrors ``misc/random.md`` — it can't be named ``random.py``
    because that shadows the stdlib ``random`` module (circular import when run).

See the doc-testing plan for the full workflow.
"""

from __future__ import annotations

import runpy
from pathlib import Path

import pytest

import radiate as rd

# Plotting snippets must not try to display anything during tests: force a headless
# backend AND make `plt.show()` a no-op. Both `events.py` (`plt.show()`) and the built-in
# `MetricCollector.plot()` in radiate call `matplotlib.pyplot.show()` at call time, so
# neutralizing it here covers every plotting snippet — the plotting logic still runs (and
# stays tested) but nothing is rendered/displayed, avoiding the Agg "cannot be shown" warning.
try:
    import matplotlib  # type: ignore[import]

    matplotlib.use("Agg")
    import matplotlib.pyplot as _plt  # type: ignore[import]

    _plt.show = lambda *args, **kwargs: None
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
