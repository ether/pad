#!/bin/sh

export JAVA_HOME="/usr/java/jdk1.6.0_14"
export SCALA_HOME="/sw/scala"
export JAVA="$JAVA_HOME/bin/java"
export SCALA="$SCALA_HOME/bin/scala"
export PATH="$JAVA_HOME/bin:$SCALA_HOME/bin:/usr/local/mysql/bin:$PATH"
export PG_CONNECTOR_JAR="/home/etherpad/etherpad-postgres/infrastructure/postgresql-8.4-701.jdbc4.jar"
