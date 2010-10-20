#!/bin/bash
# This script requires bash 3 and curl to work

# This script will check if your Etherpad instance is up and running.
# If all is well it will touch a file to keep track of when all was last OK,
# as well as also tagging that time in a Javascript source file to be used with
# the error-handler in contrib/50x-error-handler.
#
# The error-handler is a simple countdown to the automatic restart set by
# this script and will also automatically reload the page for the user at a
# random intervall after the Etherpad instance should've been restarted.
# Example directives for apache and nginx in contrib/
# Also remember to copy the HTML-file in contrib/50x-error-handler

# Remember to set the URL to the URL you want to use to check if Etherpad is up
URL=http://etherpad.your-domain.tld/ep/admin/auth
EXPECTED_CODE=200
TEST_FILE=/tmp/pad-watcher
TIMEOUT=480 # 8 minutes
RESTART_COMMAND='/opt/etherpad/bin/restart.sh &'
LAST_UPDATED_JS='/opt/etherpad/etherpad/data/50x-error-handler/last-updated.js'


###
# This is used for an error-handler for gateway errors
# If set correctly at your proxy server a seconds since last working connect
# and a timer for automatic restart will be shown.
function set_last_updated {
    echo "var last_updated = `date +%s`;" > $LAST_UPDATED_JS
    echo "var timeout = ${TIMEOUT};" >> $LAST_UPDATED_JS
}

###
# Actual test
#
# If it has been longer than TIMEOUT seconds 
# since TEST_FILE was touched RESTART_COMMAND is run.
test=`/usr/bin/curl -m 60 -s -S -I ${URL} | grep '^HTTP'`
if [[ "${test}" =~ "${EXPECTED_CODE}" ]] ; then
    touch $TEST_FILE
    set_last_updated
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
        set_last_updated
    fi
fi
