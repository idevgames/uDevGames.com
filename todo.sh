#!/bin/sh

set -x xtrace

echo "these are all the items you marked with TODO:"
grep --color TODO **/*.rs
