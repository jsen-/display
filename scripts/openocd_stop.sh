#!/bin/bash
set -eum

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && cd .. && pwd )"
PIDFILE_OCD="${DIR}/target/ocd.pid"

OCD_PID=$(<"${PIDFILE_OCD}") || true
if [[ "${OCD_PID}" != "" ]] && kill -0 "${OCD_PID}" >/dev/null 2>&1 && [[ $(ps --no-headers --format "ucmd" --pid "${OCD_PID}") == "openocd" ]]; then
    kill "${OCD_PID}" >/dev/null 2>&1
fi
