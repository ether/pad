#!/bin/bash

base=`dirname $0`
env=$base/env.sh

if [[ ! -f $env ]]; then
  echo "You need to copy $base/env.sh.template to $env and set proper values."
  exit 2
fi

. $env

pushd $base/etherpad >& /dev/null
# Since values in the properties file are relative, we have
# to start everything from the base etherpad directory.
./bin/run-local.sh
popd >& /dev/null
