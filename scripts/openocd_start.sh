#!/bin/bash
set -euxm

RUSTC_COMMIT_HASH=$(rustc --verbose --version | grep commit-hash | awk '{print $2}')
SYSROOT=$(rustc --print sysroot)
DEBUGGEE=$(cargo build --message-format=json | jq -r 'select(.executable != null).executable')
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
SUMFILE="${DIR}/target/sha1"
PIDFILE_GDB="${DIR}/target/gdb.pid"
PIDFILE_OCD="${DIR}/target/ocd.pid"

"${DIR}/scripts/openocd_stop.sh"

openocd -d1 -f "interface/stlink.cfg" -f "target/stm32f1x.cfg" >/dev/null 2>&1 &
job_openocd="$!"
echo "${job_openocd}" > "${PIDFILE_OCD}"

SUM=$(sha1sum "${DEBUGGEE}")
SAVED=$(<"${SUMFILE}") || true
if [[ "${SUM}" != "${SAVED}" ]]; then
    echo "Uploading firmware..."
    gdb --quiet --init-eval-command="file ${DEBUGGEE}" --init-eval-command="set remotetimeout 100000" --init-eval-command="target extended-remote localhost:3333" --init-eval-command="load" --batch
    echo "${SUM}" > "${SUMFILE}"
else
    echo "firmware is up to date..."
fi
