#! /bin/bash

#####
# You have to change following lines to your requirements:
#
if [ `uname` == "FreeBSD" ]; then
        export JAVA_HOME="/usr/local/jdk1.6.0"
        export SCALA_HOME="/usr/local/share/scala-2.7.7"
        export JAVA="$JAVA_HOME/bin/java"
        export SCALA="$SCALA_HOME/bin/scala"
        export SCALA_LIBRARY_JAR="/usr/local/share/scala-2.7.7/lib/scala-library.jar"
        export PATH="$JAVA_HOME/bin:$SCALA_HOME/bin:/usr/local/mysql/bin:$PATH"
        export MYSQL_CONNECTOR_JAR="/usr/local/share/java/classes/mysql-connector-java.jar"
else
        export JAVA_HOME="/usr/lib/jvm/java-6-sun/"
        export SCALA_HOME="/usr/share/java"
        export SCALA_LIBRARY_JAR="/usr/share/java/scala-library.jar"
        export MYSQL_CONNECTOR_JAR="/usr/share/java/mysql-connector-java.jar"
        export JAVA="/usr/lib/jvm/java-6-sun/bin/java"
        export SCALA="/usr/bin/scala"
        export PATH="/usr/lib/jvm/java-6-sun/bin:$PATH"
fi
#####

cd etherpad
exec bin/run-local.sh
