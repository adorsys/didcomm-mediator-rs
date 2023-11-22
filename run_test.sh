#!/bin/bash
for dir in ./did-utils/ ./did-endpoint/ ./generic-server/ ./mediator-coordination/
do
  cd "${dir}"
  if [ -f Cargo.toml ]; then
    echo "Running tests in: ${dir}"
    cargo test
  fi
  cd ..
done
