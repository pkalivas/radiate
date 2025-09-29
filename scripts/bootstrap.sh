#!/usr/bin/env bash
set -euo pipefail

# Load env and optional overrides
. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/env.sh"

BUILD_WHEEL="${SCRIPT_ROOT}/build_wheel.sh"

"$BUILD_WHEEL" --gil --python "$DEFAULT_PYTHON_VERSION" --venv-dir '.venv' 
