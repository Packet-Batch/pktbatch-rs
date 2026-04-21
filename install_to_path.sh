#!/bin/bash

TYPE=release
OUT=pktbatch

cp -f ./target/$TYPE/pktbatch-rs /usr/bin/$OUT

echo "Installed pktbatch to /usr/bin/$OUT"