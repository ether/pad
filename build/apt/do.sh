#!/bin/bash

################################################################################
#
# Copyright (c) 2010 penSec.IT UG (haftungsbeschränkt)
#        http://www.pensec.it
#        mail@pensec.it
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
#
# To build a Debian/Ubuntu package simply run this script. It will prompt you
# for the repository to build from.
#
# Dependencies: debconf (>= 0.2.17), dpkg, sun-java6-jdk, scala, mysql-client,
#     libmysql-java, bash, mercurial | git-core
#
#####



#####
#
# There has to be a better way of doing this ;-)
#
REPOSITORY_0_URL="https://etherpad.googlecode.com/hg/trunk"
REPOSITORY_0_NAME="deprecated"
REPOSITORY_0_TYPE="hg"
REPOSITORY_1_URL="git://github.com/ether/pad.git"
REPOSITORY_1_NAME="official"
REPOSITORY_1_TYPE="git"
REPOSITORY_2_URL="git://github.com/redhog/pad.git"
REPOSITORY_2_NAME="devel"
REPOSITORY_2_TYPE="git"
REPOSITORY_3_URL="git://github.com/johnyma22/pad.git"
REPOSITORY_3_NAME="johny"
REPOSITORY_3_TYPE="git"
REPOSITORY_4_URL="git://github.com/Pita/pad.git"
REPOSITORY_4_NAME="pita"
REPOSITORY_4_TYPE="git"
REPOSITORY_5_URL="./../../"
REPOSITORY_5_NAME="local"
REPOSITORY_5_TYPE="git"


# Tell the user his choises
echo "Please choose the repository to build a debian package from:"
echo ""

for i in {0..5}
do
	URL="REPOSITORY_${i}_URL"
	NAME="REPOSITORY_${i}_NAME"
	TYPE="REPOSITORY_${i}_TYPE"

	echo "	${i})	${!NAME}	${!URL}"
done
echo ""
read -p "Repository id: " REPOSITORY



# Test if user's too stupid to input correct number
URL="REPOSITORY_${REPOSITORY}_URL"
NAME="REPOSITORY_${REPOSITORY}_NAME"
TYPE="REPOSITORY_${REPOSITORY}_TYPE"

if [ "" == "${!URL}" ]; then
	echo "Invalid repository id \"${REPOSITORY}\""
	exit 1
fi


# Communicate repository details to build script
export REPOSITORY_URL="${!URL}"
export REPOSITORY_NAME="${!NAME}"
export REPOSITORY_TYPE="${!TYPE}"
#
#####



#####
#
# Is a complete rebuild necessary or is a simple update sufficient
#
echo ""
read -p "Purge before rebuild (yes/no) [n]: " REBUILD

if [ "y" == "${REBUILD}" ]; then
	echo "You answered \"${REBUILD}\", will purge before rebuilding"
	REBUILD="rebuild"
else
	echo "You answered \"${REBUILD}\", will try ${!TYPE} pull before rebuilding"
fi
#
#####



#####
#
# Tell user what we're going to to and then do it
#
echo ""
echo "Will build package etherpad-${!NAME} form ${!TYPE}:${!URL}, please be patient..."
echo ""

bash -c "./build/build.sh ${REBUILD}"
#
#####

