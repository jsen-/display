#!/bin/bash
set -eumo pipefail

RUSTC_COMMIT_HASH=$(rustc --verbose --version | grep commit-hash | awk '{print $2}')
SYSROOT=$(rustc --print sysroot)
gdb --init-eval-command="set substitute-path /rustc/${RUSTC_COMMIT_HASH}/ ${SYSROOT}/lib/rustlib/src/rust/" "$@"