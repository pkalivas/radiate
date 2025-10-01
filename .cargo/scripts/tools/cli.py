# cli.py
from dataclasses import dataclass

@dataclass
class Norm:
    cmd: str  # "dev" | "wheel" | "test" | "ex"
    py: str = None  # interpreter spec/path
    name: str = None  # for ex
    rest: list[str] = None  # passthrough after "--"


def _split_passthrough(raw_argv: list[str]) -> list[str]:
    # everything after the first '--' is passthrough
    if "--" in raw_argv:
        i = raw_argv.index("--")
        return raw_argv[i + 1 :]
    return []


def normalize_args(args, raw_argv: list[str]) -> Norm:
    """
    args: parsed Namespace from build_parser()
    raw_argv: sys.argv[1:] (to capture passthrough even for shortcuts)
    """
    rest = _split_passthrough(raw_argv)
    # 1) real subcommands take precedence
    if args.cmd == "dev":
        return Norm(
            cmd="dev",
            py=getattr(args, "py", None),
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "wheel":
        return Norm(
            cmd="wheel",
            py=getattr(args, "py", None),
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "test":
        return Norm(
            cmd="test",
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "ex":
        return Norm(
            cmd="ex",
            name=args.name,
            rest=getattr(args, "rest", []) or rest,
        )
    if args.cmd == "clean":
        return Norm(
            cmd="clean",
            rest=getattr(args, "rest", []) or rest,
        )

    # 2) shortcuts (only one can be present due to mutual-exclusion)
    if args.dev is not None:
        py = args.dev or None  # empty string means "use default"
        return Norm(cmd="dev", py=py, rest=rest)
    if args.wheel is not None:
        py = args.wheel or None
        return Norm(cmd="wheel", py=py, rest=rest)
    if args.test:
        return Norm(cmd="test", rest=rest)
    if args.ex:
        return Norm(cmd="ex", name=args.ex, rest=rest)
    if args.clean:
        return Norm(cmd="clean", rest=rest)

    raise SystemExit(2)