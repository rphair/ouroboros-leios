#!/usr/bin/env bash

set -e

cd "$(dirname "${BASH_SOURCE[0]}")"

echo "SCENARIO: haskell | 600 | default | 100-nodes | 25.0 | 98304 | 1.0 | 20"

if [[ ! -p sim.log ]]
then
  mkfifo sim.log
fi

cleanup() {
  if [[ -p sim.log ]]
  then
    rm sim.log
  fi
}

trap cleanup EXIT

mongoimport --quiet --host "$HOST" --db "$DB" --collection "RAW_b15e68af" --drop sim.log &

../../ols sim short-leios --leios-config-file config.json --topology-file network.yaml --output-file sim.log --output-seconds 600 > stdout 2> stderr

echo "SCENARIO: haskell | 600 | default | 100-nodes | 25.0 | 98304 | 1.0 | 20"

mongo --quiet --host "$HOST" "$DB" scenario.js "../../queries/import.js"
