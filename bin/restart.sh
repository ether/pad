#!/bin/bash

CURRDIR=`dirname "$0"`

$CURRDIR/stop.sh
if [[ $? -eq 0 ]] ; then
	$CURRDIR/run.sh
	exit $?
else
	exit 1
fi
