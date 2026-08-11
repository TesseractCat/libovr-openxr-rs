#!/usr/bin/env bash
# Start the headless Monado runtime and launch Echo with the same environment.
set -euo pipefail

project_root=$(cd "$(dirname "$0")/.." && pwd)

"$project_root/tools/run-monado-null.sh"
# shellcheck disable=SC1091
source "$project_root/artifacts/monado-null/env.sh"

# The disposable Proton prefix is more reliable without fsync during repeated
# injector/debug launches. run-game.sh builds and deploys the current DLLs.
export WINEFSYNC="${WINEFSYNC:-0}"
exec "$project_root/tools/run-game.sh" "$@"
