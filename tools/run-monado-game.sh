#!/usr/bin/env bash
# Start the headless Monado runtime and launch Echo with the same environment.
set -euo pipefail

project_root=$(cd "$(dirname "$0")/.." && pwd)

"$project_root/tools/run-monado-null.sh"
# shellcheck disable=SC1091
source "$project_root/artifacts/monado-null/env.sh"

# run-game.sh applies the shared Proton/xrizer launch environment after this
# wrapper selects the disposable Monado runtime.
exec "$project_root/tools/run-game.sh" "$@"
