#!/usr/bin/env python3
import argparse
import sys

from tools.action import (
    cmd_dev,
    cmd_wheel,
    cmd_test,
    cmd_ex,
    cmd_clean,
)
from tools.cli import normalize_args


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="cargo py",
        description="Radiate Python helper (uv + maturin + pytest + examples)",
    )

    # shortcuts (mutually exclusive)
    g = p.add_mutually_exclusive_group()
    g.add_argument(
        "--dev",
        nargs="?",
        const="",
        metavar="PY",
        help="shortcut for dev (optional PY, e.g. 3.13t)",
    )
    g.add_argument(
        "--wheel",
        nargs="?",
        const="",
        metavar="PY",
        help="shortcut for wheel (optional PY)",
    )
    g.add_argument("--test", action="store_true", help="shortcut for test")
    g.add_argument("--ex", metavar="NAME", help="shortcut for example name/path")
    g.add_argument(
        "--clean", action="store_true", help="clean build artifacts before action"
    )

    # subcommands (exact same behavior)
    sub = p.add_subparsers(dest="cmd")

    s_dev = sub.add_parser("dev", help="maturin develop (editable)")
    s_dev.add_argument("--py", help="Python spec (e.g. 3.12, 3.13t)")
    s_dev.add_argument("rest", nargs=argparse.REMAINDER)

    s_wheel = sub.add_parser("wheel", help="maturin build (wheel)")
    s_wheel.add_argument("--py", help="Python spec")
    s_wheel.add_argument("rest", nargs=argparse.REMAINDER)

    s_test = sub.add_parser("test", help="pytest via uv")
    s_test.add_argument("rest", nargs=argparse.REMAINDER)

    s_ex = sub.add_parser("ex", help="run example from py-radiate/examples")
    s_ex.add_argument("name", help="example name or path")
    s_ex.add_argument("rest", nargs=argparse.REMAINDER)

    s_clean = sub.add_parser("clean", help="clean build artifacts")
    s_clean.add_argument("rest", nargs=argparse.REMAINDER)

    return p


def main(argv=None):
    # print(argv)
    # return
    argv = sys.argv[1:] if argv is None else argv
    parser = build_parser()
    args = parser.parse_args(argv)
    norm = normalize_args(args, argv)

    if norm.cmd == "dev":
        return cmd_dev(norm.py, norm.rest or [])
    if norm.cmd == "wheel":
        return cmd_wheel(norm.py, norm.rest or [])
    if norm.cmd == "test":
        return cmd_test(norm.rest or [])
    if norm.cmd == "ex":
        return cmd_ex(norm.name, norm.rest or [])
    if norm.cmd == "clean":
        return cmd_clean(norm.rest or [])

    parser.print_help()
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
