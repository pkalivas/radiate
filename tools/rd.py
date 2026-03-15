import subprocess
import sys

cmd = sys.argv[1:]

if not cmd:
    print("rd examples | docs")
    sys.exit(1)

match cmd[0]:
    case "examples":
        subprocess.run(["uv", "run", "python", "tools/examples.py", *cmd[1:]])
    case "docs":
        subprocess.run(["mkdocs", *cmd[1:]])
    case _:
        print("unknown command")
