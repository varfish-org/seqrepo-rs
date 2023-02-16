#!/usr/bin/bash

# Setup Logging -------------------------------------------------------------

log()
{
    >&2 echo $@
}

debug()
{
    [[ "${VERBOSE-0}" -ne 0 ]] && >&2 echo $@
}

set -euo pipefail

if [[ "${VERBOSE-0}" -ne 0 ]]; then
    set -x
fi

# Initialization ------------------------------------------------------------

if [[ "$#" -ne 0 ]]; then
    log "USAGE: bootstrap.sh"
    log ""
    log "Set VERBOSE=1 to increase verbosity."
    exit 1
fi

# path to the directory where the script resides.
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# Destination directory.
DST=$SCRIPT_DIR

# Name of database file.
DB=aliases.sqlite3

# Import SQLite database ----------------------------------------------------

cd $DST
rm -f $DB
cat schema.sql data.sql | sqlite3 $DB
