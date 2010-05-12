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
#####

cd etherpad
exec bin/run-local.sh "$@" >> "$LOG" 2>&1

