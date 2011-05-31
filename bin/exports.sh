#!/bin/bash

################################################################################
#
# Copyright (c) 2010 penSec.IT UG (haftungsbeschränkt)
#        http://www.pensec.it
#        mail@pensec.it
# Copyright (c) 2010 Egil Möller <egil.moller@piratpartiet.se>
# 
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at
# 
#        http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License. 
#
################################################################################





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
elif [[ $(uname -s) == CYGWIN* ]]; then
        export JAVA_HOME="C:\Etherpad\JDK1.6_23"
        export SCALA_HOME="C:\Etherpad\scala-2.7.4.final"
        export JAVA="$JAVA_HOME\bin\java"
        export SCALA="$SCALA_HOME\bin\scala"
        export SCALA_LIBRARY_JAR="$SCALA_HOME\lib\scala-library.jar"
        export PATH="$JAVA_HOME\bin:$SCALA_HOME\bin:$PATH"
        export MYSQL_CONNECTOR_JAR="C:\Etherpad\mysql-connector-java-5.1.16\mysql-connector-java-5.1.16-bin.jar"
else
        [ -e "/usr/lib/jvm/java-6-openjdk" ] && export JAVA_HOME="/usr/lib/jvm/java-6-openjdk"
        [ -e "/usr/lib/jvm/java-6-sun" ] && export JAVA_HOME="/usr/lib/jvm/java-6-sun"
        [ -e "/usr/lib/jvm/java-6-openjdk" ] && export JAVA_OPTS="-Xbootclasspath/p:../infrastructure/lib/rhino-js-1.7r1.jar:/usr/share/java/scala-library.jar" 
        export SCALA_HOME="/usr/share/java"
        export SCALA_LIBRARY_JAR="/usr/share/java/scala-library.jar"
        export MYSQL_CONNECTOR_JAR="/usr/share/java/mysql-connector-java.jar"
        export JAVA="$JAVA_HOME/bin/java"
        export SCALA="/usr/bin/scala"
        export PATH="$JAVA_HOME/bin:$PATH"
fi

if ! [ -e "$MYSQL_CONNECTOR_JAR" ]; then
        echo "MySql Connector jar '$MYSQL_CONNECTOR_JAR' not found - Download it here: http://dev.mysql.com/downloads/connector/j/3.1.html"
        exit 1
fi

if ! [ -e "$SCALA_LIBRARY_JAR" ]; then
        echo "Scala Library cannot be found '$SCALA_LIBRARY_JAR' not found - Download it here: http://www.scala-lang.org/"
        exit 1
fi

if ! [ -e "$JAVA" ]; then
        echo "Java cannot be found '$JAVA' not found - Download it here: http://openjdk.java.net/"
        exit 1
fi

if ! [ -e "$SCALA" ]; then
        echo "Scala cannot be found '$SCALA' not found - Download it here: http://www.scala-lang.org/"
        exit 1
fi
