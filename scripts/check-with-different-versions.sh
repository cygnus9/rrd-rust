#!/bin/bash

set -exu

cd $(dirname $0)/..

# Make sure the wrapper handles the different *mut/*const expectations
# in `graph` from different librrd versions.

# fedora uses 1.8 and 1.9
for FVERS in 39 40 41 42; do
  FTAG="rrd-fedora-$FVERS"
  docker build --build-arg "fedora_version=$FVERS" -t "$FTAG" -f rrd-fedora.Dockerfile .
  docker run --rm $FTAG
done

# ubuntu uses all librrd 1.7.2
for UVERS in 20.04 22.04 24.04 24.10 25.04; do
  UTAG="rrd-ubuntu-$UVERS"
  docker build --build-arg "ubuntu_version=$UVERS" -t "$UTAG" -f rrd-ubuntu.Dockerfile .
  docker run --rm $UTAG
done

# unfortunately we don't have a great way to try alpine (rrdtool 1.9)
# as clang-sys really doesn't like alpine's clang