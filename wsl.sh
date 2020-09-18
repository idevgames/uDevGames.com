#!/bin/sh

# wsl2 is a little special because it's not port-forwarded (how would it know
# what ports to forward?) This is a little shortcut to finding the current
# private ip and starting the development server to bind to that so you can hit
# it from the host's web browser.

set -x

cargo build

if [ $? -neq 0 ]; then
  echo "Fix your broken build, man."
  return -1
fi

export UDEVGAMES_APP_ADDRESS=$(hostname -I)
cargo run -- $@
