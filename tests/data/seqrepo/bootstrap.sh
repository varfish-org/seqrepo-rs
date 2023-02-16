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

if [[ "$#" -ne 2 ]]; then
    log "USAGE: bootstrap.sh SEQREPO INSTANCE"
    log ""
    log "Set VERBOSE=1 to increase verbosity."
    exit 1
fi

# path to the directory where the script resides.
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# SeqRepo source directory.
SRC=$1

# SeqRepo instance.
INSTANCE=$2

# Destination directory.
DST=$SCRIPT_DIR

# Import SQLite database ----------------------------------------------------

rm -rf $DST

seqrepo --root-directory $DST init --instance-name latest

tx=NM_001304430.2  # hash="5q5HZTCRudL17NTiv5Bn6th__0FrZH04"

echo ">$tx" \
> $DST/$tx.fasta

echo "sr[\"$tx\"][:]" \
| seqrepo --root-directory $SRC start-shell -i $INSTANCE \
| grep Out \
| awk '{ print $NF }' \
| tr -d "'" \
>> $DST/$tx.fasta

seqrepo --root-directory $DST load --instance-name latest --namespace refseq $DST/$tx.fasta
