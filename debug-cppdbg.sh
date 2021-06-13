#!/bin/bash

set -eum
RUSTC_COMMIT_HASH=$(rustc --verbose --version | grep commit-hash | awk '{print $2}')
SYSROOT=$(rustc --print sysroot)
DEBUGGEE=$(cargo build --message-format=json | jq -r 'select(.executable != null).executable')
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SUMFILE="${DIR}/target/sha1"
PIDFILE_GDB="${DIR}/target/gdb.pid"
PIDFILE_OCD="${DIR}/target/ocd.pid"

function cleanup {
    echo "killing openocd" >&2
    kill "${job_openocd}"

    if [ ! -z ${job_gdb+x} ]; then
        echo "killing gdb" >&2
        kill "${job_gdb}" >/dev/null 2>&1 || true
    fi
}

GDB_PID=$(<"${PIDFILE_GDB}") || true
if [[ "${GDB_PID}" != "" ]] && kill -0 "${GDB_PID}" >/dev/null 2>&1 && [[ $(ps --no-headers --format "ucmd" --pid "${GDB_PID}") == "gdb" ]]; then
    kill "${GDB_PID}" >/dev/null 2>&1
fi
OCD_PID=$(<"${PIDFILE_OCD}") || true
if [[ "${OCD_PID}" != "" ]] && kill -0 "${OCD_PID}" >/dev/null 2>&1 && [[ $(ps --no-headers --format "ucmd" --pid "${OCD_PID}") == "openocd" ]]; then
    kill "${OCD_PID}" >/dev/null 2>&1
fi

openocd -d1 -f "interface/stlink.cfg" -f "target/stm32f1x.cfg" >/dev/null 2>&1  &
job_openocd="$!"
echo "${job_openocd}" > "${PIDFILE_OCD}"
trap "cleanup" EXIT

SUM=$(sha1sum "${DEBUGGEE}")
SAVED=$(<"${SUMFILE}") || true
if [[ "${SUM}" != "${SAVED}" ]]; then
    echo "Uploading firmware..."
    gdb --quiet --init-eval-command="file ${DEBUGGEE}" --init-eval-command="set remotetimeout 100000" --init-eval-command="target extended-remote localhost:3333" --init-eval-command="load" --batch
    echo "${SUM}" > "${SUMFILE}"
else
    echo "firmware is up to date..."
fi

gdb --init-eval-command="file ${DEBUGGEE}" \
  --init-eval-command="set substitute-path /cargo ${CARGO_HOME}" \
  --init-eval-command="set substitute-path asm ${CARGO_HOME}/registry/src/github.com-1ecc6299db9ec823/cortex-m-0.7.1/asm" \
  --init-eval-command="set substitute-path /rustc/${RUSTC_COMMIT_HASH}/ ${SYSROOT}/lib/rustlib/src/rust/" \
  -q -x "${DIR}/debug.gdb" "$@" &

job_gdb="$!"
trap "cleanup" EXIT
echo "${job_gdb}" > "${PIDFILE_GDB}"
fg
