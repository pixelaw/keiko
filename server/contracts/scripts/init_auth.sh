#!/bin/bash
set -euo pipefail
pushd $(dirname "$0")/..

# enable system -> component authorizations
COMPONENTS=("Position" "Moves" )

for component in ${COMPONENTS[@]}; do
    sozo auth writer $component $ACTIONS
done

for component in ${COMPONENTS[@]}; do
    sozo auth writer $component $ACTIONS
done

echo "Default authorizations have been successfully set."