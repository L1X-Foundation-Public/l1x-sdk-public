#!/bin/bash

set -xe

packages=("l1x-sys" "l1x-sdk-macro" "l1x-sdk")

for package in "${packages[@]}"; do
    sleep 2
    cargo publish -p "${package}"
    # Sleep a bit to let the previous package upload to crates.io. Otherwise we fail publishing checks.
    sleep 30
done