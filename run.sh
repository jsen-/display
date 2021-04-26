#!/bin/bash

set -eum
DEBUGGEE=$(cargo build --release --message-format=json "$@" | jq -r 'select(.executable != null).executable')
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SUMFILE="$DIR/target/sha1"
PIDFILE_GDB="$DIR/target/gdb.pid"
PIDFILE_OCD="$DIR/target/openocd.pid"

function cleanup {
    echo "killing openocd" >&2
    kill $job_openocd
}

GDB_PID=$(<"$PIDFILE_GDB") || true
if [[ "$GDB_PID" != "" ]] && kill -0 "$GDB_PID" >/dev/null 2>&1 && [[ $(ps --no-headers --format "ucmd" --pid "$GDB_PID") == "gdb" ]]; then
    kill "$GDB_PID" >/dev/null 2>&1
fi
OCD_PID=$(<"$PIDFILE_OCD") || true
if [[ "$OCD_PID" != "" ]] && kill -0 "$OCD_PID" >/dev/null 2>&1 && [[ $(ps --no-headers --format "ucmd" --pid "$OCD_PID") == "openocd" ]]; then
    kill "$OCD_PID" >/dev/null 2>&1
fi

openocd >&2 &
job_openocd="$!"
echo $job_openocd > "$PIDFILE_OCD"
trap "cleanup" EXIT

SUM=$(sha1sum "$DEBUGGEE")
SAVED=$(<"$SUMFILE") || true
if [[ "$SUM" != "$SAVED" ]]; then
    echo "Uploading firmware..."
    gdb --quiet --init-eval-command="file ${DEBUGGEE}" --init-eval-command="set remotetimeout 100000" --init-eval-command="target extended-remote localhost:3333" --init-eval-command="load" --batch
    echo "$SUM" > $SUMFILE
else
    echo "firmware is up to date..."
fi

gdb --init-eval-command="file ${DEBUGGEE}" -q -x "${DIR}/run.gdb" "$@"