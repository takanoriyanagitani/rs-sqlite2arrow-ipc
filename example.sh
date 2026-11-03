#!/bin/bash

set -u

wsm="./sqlite2ipc.wasm"

hdir="$PWD"
gdir="/guest.d"
iname="in.sqlite.db3"

hpath="${hdir}/${iname}"
gpath="${gdir}/${iname}"

select_all="
  SELECT
    timestamp,
    severity,
    body
  FROM log
  ORDER BY rowid
"

run_wasmtime(){
    wasmtime \
      run \
      --env ENV_SQL_SELECT_ALL="${select_all}" \
      --env ENV_ALLOW_NULL=true \
      --env ENV_BAT_SIZE=8192 \
      --env ENV_SQLITE_DB_FILE_PATH="${gpath}" \
      --dir "${hdir}::${gdir}" \
      "${wsm}"
}

run_wasi(){
  run_wasmtime
}

geninput(){
  echo generating input sqlite file...

  echo "
    CREATE TABLE IF NOT EXISTS log (
      id INTEGER PRIMARY KEY,
      timestamp INTEGER NOT NULL,
      severity TEXT NOT NULL,
      body TEXT NOT NULL
    )
  " | sqlite3 "${hpath}"

  inst_prefix='INSERT INTO log(timestamp, severity, body)'

  echo "${inst_prefix} VALUES(1,'INFO','apt update done')" | sqlite3 "${hpath}"
  echo "${inst_prefix} VALUES(2,'WARN','apt upd failure')" | sqlite3 "${hpath}"

  echo
}

print_ipc(){
  imports=""
  imports="${imports}import functools;"
  imports="${imports}import sys;"
  imports="${imports}import pyarrow as pa;"

  cat /dev/stdin |
    python3 -c "${imports}"'functools.reduce(
      lambda state,f: f(state),
      [
        pa.ipc.open_stream,
        next,
        print,
      ],
      sys.stdin.buffer,
    )'
}

test -f "${hpath}" || geninput

echo printing the input sqlite...
echo "${select_all}" | sqlite3 -markdown "${hpath}"
echo

echo converting to ipc stream...
run_wasi | print_ipc
