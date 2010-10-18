#!/bin/bash
# requires bash 3

URL=http://etherpad-site.tld/ep/admin/auth
EXPECTED_CODE=200
TEST_FILE=/tmp/pad-watcher
TIMEOUT=480 # 5 minutes
RESTART_COMMAND='/opt/etherpad/bin/restart.sh &'

###
# Actual test
#
# If it has been longer than TIMEOUT seconds 
# since TEST_FILE was touched RESTART_COMMAND is run.

test=`/usr/bin/curl -m 60 -s -S -I ${URL} | grep '^HTTP'`
if [[ "${test}" =~ "${EXPECTED_CODE}" ]] ; then
    touch $TEST_FILE
else
    if [[ -f $TEST_FILE ]] ; then
        NOW=`date +%s` # Seconds since epoch
        FILE_TIMEOUT=`stat -c %Z ${TEST_FILE}` # Last changed since epoch

        DIFF=`expr ${NOW} - ${FILE_TIMEOUT}`
        if [[ $DIFF -gt $TIMEOUT ]] ; then
           logger "[${0}] About to run: '${RESTART_COMMAND}'"
           touch $TEST_FILE
           echo "Restarting `date`" >> /tmp/etherpad-watcher.log
           $RESTART_COMMAND
        fi
    else
        touch $TEST_FILE
    fi
fi
