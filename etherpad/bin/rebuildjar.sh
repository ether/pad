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
            echo "fastjar version $version can't build etherpad.  Falling back to standard jar."
            JAR=jar
        else
            JAR=fastjar
        fi
    else
        JAR=jar
    fi
fi

function depscheck {
    if [ ! -d "$JAVA_HOME" ]; then
	echo "\$JAVA_HOME does not point to an existing dir; should be e.g. /usr/java/latest"
	exit 1
    fi
    if [ ! -d "$SCALA_HOME" ]; then
	echo "\$SCALA_HOME does not point to an existing dir; should be e.g. /usr/share/scala"
	exit 1
    fi
    if [ ! -e "$SCALA" ]; then
	echo "\$SCALA does not point to an existing file; should be e.g. /usr/bin/scala"
	exit 1
    fi
    if [ ! -e "$JAVA" ]; then
	echo "\$JAVA does not point to an existing file; should be e.g. /usr/bin/java"
	exit 1
    fi
    if [ ! -e "$MYSQL_CONNECTOR_JAR" ]; then
        echo "\$MYSQL_CONNECTOR_JAR does not point to an existing file; should be e.g. /usr/share/java/mysql-connector-java.jar"
        exit 1
    fi

}

depscheck

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
