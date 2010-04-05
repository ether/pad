#!/bin/bash -e

#  Copyright 2009 Google Inc.
#  
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#  
#       http://www.apache.org/licenses/LICENSE-2.0
#  
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS-IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

bin/java-version.sh

if [ -z "$JAR" ]; then
    if [ ! -z $(which fastjar 2>/dev/null) ]; then
        # http://lists.gnu.org/archive/html/fastjar-dev/2009-12/msg00000.html
        version=`fastjar --version | grep fastjar | sed 's/.* //g'`
        if [[ "$version" = "0.97" || "$version" = "0.98" ]]; then
            echo "fastjar version $version can't build EtherPad.  Falling back to standard jar."
            JAR=jar
        else
            JAR=fastjar
        fi
    else
        JAR=jar
    fi
fi

[ -z "$JAVA_HOME" ] && read -p "\$JAVA_HOME is not set, please enter the path to your Java installation: " JAVA_HOME
if [ ! -e "$JAVA_HOME" ]; then
    echo "The path to \$JAVA_HOME ($JAVA_HOME) does not exist, please check and try again."
    exit 1
else
    export JAVA_HOME
fi

[ -z "$SCALA_HOME" ] && read -p "\$SCALA_HOME is not set, please enter the path to your Scala installation: " SCALA_HOME
if [ ! -e "$SCALA_HOME" ]; then
    echo "The path to \$SCALA_HOME ($SCALA_HOME) does not exist, please check and try again."
    exit 1
else
    export SCALA_HOME
fi

if [ -z "$SCALA" ]; then
    if [ `which scala 2>/dev/null 1>/dev/null` ]; then
        SCALA=`which scala`
        echo "Using 'scala' binary found at $SCALA. Set \$SCALA to use another one."
    elif [ -x "$SCALA_HOME/bin/scala" ]; then
        SCALA="$SCALA_HOME/bin/scala"
        echo "Using 'scala' binary found at $SCALA. Set \$SCALA to use another one."
    else
        read -p "\$SCALA is not set and the 'scala' binary could not be found, please enter the path to the file: " SCALA
    fi
fi
if [ ! -x "$SCALA" ]; then
    echo "The path to \$SCALA ($SCALA) is not an executable file, please check and try again."
    exit 1
else
    export SCALA
fi

if [ -z "$JAVA" ]; then
    if [ `which java 2>/dev/null 1>/dev/null` ]; then
        JAVA=`which java`
        echo "Using 'java' binary found at $JAVA. Set \$JAVA to use another one."
    elif [ -x "$JAVA_HOME/bin/java" ]; then
        JAVA="$JAVA_HOME/bin/java"
        echo "Using 'java' binary found at $JAVA. Set \$JAVA to use another one."
    else
        read -p "\$JAVA is not set and the 'java' binary could not be found, please enter the path to the file: " JAVA
    fi
fi
if [ ! -x "$JAVA" ]; then
    echo "The path to \$JAVA ($JAVA) is not an executeable file, please check and try again."
    exit 1
else
    export JAVA
fi

[ -z "$MYSQL_CONNECTOR_JAR" ] && read -p "\$MYSQL_CONNECTOR_JAR is not set, please enter the path to the MySQL JDBC driver .jar file: " MYSQL_CONNECTOR_JAR
if [ ! -e "$MYSQL_CONNECTOR_JAR" ]; then
    echo "The path to \$MYSQL_CONNECTOR_JAR ($MYSQL_CONNECTOR_JAR) does not exist, please check and try again."
    exit 1
else
    export MYSQL_CONNECTOR_JAR
fi

# Check for javac version. Unfortunately, javac doesn't tell you whether
# it's Sun Java or OpenJDK, but the "java" binary that's in the same
# directory will.
if [ -e "$JAVA_HOME/bin/java" ]; then
    ($JAVA_HOME/bin/java -version 2>&1) | {
        while read file; do
            javaver=$file
        done
        for word in $javaver; do
            if [ $word != "Java" ]; then
                echo "$JAVA_HOME/bin/java is from a non-Sun compiler, and may not be able to compile EtherPad. If you get syntax errors, you should point \$JAVA_HOME at a Sun Java JDK installation instead."
            fi
            break
        done
    }
fi

function notify {
    if [ ! -z $(which growlnotify 2>/dev/null) ]; then
	echo $0 finished | growlnotify
    fi   
}
trap notify EXIT

source ../infrastructure/bin/compilecache.sh

suffix="-dev";
if [ "$1" == "prod" ]; then
    suffix="";
    shift;
fi

OWD=`pwd`
cd ../infrastructure
JAR=$JAR bin/makejar.sh $@

rm -rf build/etherpad-jars
mkdir -p build/etherpad-jars

echo "including etherpad JARs..."

JARFILES="echo ../etherpad/lib/*.jar"
function genjar {
    echo "unzipping JARs..."
    pushd $1 >> /dev/null

    for a in ../../../etherpad/lib/*.jar; do
	$JAR xf $a
	rm -rf META-INF/{MANIFEST.MF,NOTICE{,.txt},LICENSE{,.txt},INDEX.LIST,SUN_MICR.{RSA,SF},maven}
    done

    popd >> /dev/null    
}
cacheonfiles JAR-etherpad "$JARFILES" genjar 1

echo "updating..."

pushd buildcache/JAR-etherpad >> /dev/null
$JAR uf ../../build/appjet.jar `ls . | grep -v "^t$"`

echo "done."

popd >> /dev/null

dst="$OWD/appjet-eth$suffix.jar"
cp -f build/appjet.jar $dst
cd $OWD
echo "wrote $dst"
