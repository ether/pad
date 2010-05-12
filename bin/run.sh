#! /bin/bash

#####
# You have to change following lines to your requirements:
#
export JAVA_HOME="/usr/lib/jvm/java-6-sun/"
export SCALA_HOME=/usr/share/java
export MYSQL_CONNECTOR_JAR=/usr/share/java/mysql-connector-java.jar
export JAVA="/usr/lib/jvm/java-6-sun/bin/java"
export SCALA="/usr/bin/scala"
export PATH="/usr/lib/jvm/java-6-sun/bin:$PATH"
export LOG="/var/log/etherpad/server.log"
export SOFFICE_BIN="/usr/bin/soffice"
#####

cd etherpad
if [ -x "$SOFFICE_BIN" ]; then
	"$SOFFICE_BIN" -headless -nofirststartwizard -accept="socket,host=localhost,port=8100;urp;StarOffice.Service" &
fi
exec bin/run-local.sh --etherpad.soffice="$SOFFICE_BIN" "$@" >> "$LOG" 2>&1

