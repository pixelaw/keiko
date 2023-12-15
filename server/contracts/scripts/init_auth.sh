#!/bin/bash
set -euo pipefail
pushd $(dirname "$0")/..

source scripts/env.sh

# enable system -> component authorizations
COMPONENTS=("Position" "Moves" )

for component in ${COMPONENTS[@]}; do
    sleep 0.1
    sozo auth writer $component $DOJO_EXAMPLES_ACTIONS_ACTIONS
done

for component in ${COMPONENTS[@]}; do
    sleep 0.1
    sozo auth writer $component $DOJO_EXAMPLES_ACTIONS_ACTIONS
done

echo "Default authorizations have been successfully set."