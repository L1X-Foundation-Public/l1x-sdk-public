#!/bin/bash

set -e

cd "$(dirname "$0")"
cargo doc -p l1x-sdk -p l1x-sdk-macros --all-features --no-deps