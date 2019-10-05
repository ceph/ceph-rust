#!/bin/bash

set -e

rm -rf /tmp/ceph
mkdir /tmp/ceph
/micro-osd.sh /tmp/ceph
export CEPH_CONF=/tmp/ceph/ceph.conf

set +e
