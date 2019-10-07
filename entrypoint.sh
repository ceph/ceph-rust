#!/bin/bash

. /setup-micro-osd.sh

set -e

cargo test --features rados_striper

cargo run --features rados_striper --example rados_striper
