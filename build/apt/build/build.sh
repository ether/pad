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
# Build a Debian/Ubuntu Etherpad package of the source
#
# @param $1 (optional) If "${1}" == "rebuild" then the cached repository will be
#     purged and cloned instead of simply updated
# @param $REPOSITORY_URL the source of the package
# @param $REPOSITORY_NAME name of the new package, package will be named
#     etherpad-${REPOSITORY_NAME}
# @param $REPOSITORY_TYPE Element of set {hg, git}, describes which command line
#     toolchain (mercurial or git) will be used
#
if [ "" == "${REPOSITORY_URL}" ]; then
	echo "Missing environment variable REPOSITORY_URL"
	exit 1
fi
if [ "" == "${REPOSITORY_NAME}" ]; then
	echo "Missing environment variable REPOSITORY_NAME"
	exit 1
fi
if [ "" == "${REPOSITORY_TYPE}" ]; then
	echo "Missing environment variable REPOSITORY_TYPE"
	exit 1
fi



#####
# You have to change following lines to your requirements:
#
export JAVA_HOME=/usr/lib/jvm/java-6-sun/
export SCALA_HOME=/usr/share/java
export MYSQL_CONNECTOR_JAR=/usr/share/java/mysql-connector-java.jar
export JAVA="$JAVA_HOME/bin/java"
export SCALA="$SCALA_HOME/bin/scala"
export PATH="$JAVA_HOME/bin:$SCALA_HOME/bin:$PATH"
#
#####


#####
# Don't change this!
#
REPOSITORY="${REPOSITORY_URL}"
BRANCH="${REPOSITORY_NAME}"

TMP_DIR=".tmp.${REPOSITORY_NAME}"
BUILD_DIR=".build.${REPOSITORY_NAME}"
PACKAGE_DIR=".package.${REPOSITORY_NAME}"
REVISION_FILE=".revision.${REPOSITORY_NAME}"

REBUILD="no"
if [ "rebuild" == "${1}" ]; then
	REBUILD="yes"
fi
#
#####



# If the repository isn't checked out by now we can't do a simple {git|hg} pull,
# we need to do a complete {git|hg} clone instead
if [ "yes" != "${REBUILD}" ]; then
	if [ -d "${TMP_DIR}" ]; then
		bash -c "cd ${TMP_DIR}; ${REPOSITORY_TYPE} pull"
	else
		echo "Repository does not exist, will fetch"
		REBUILD="yes"
	fi
fi

# Refresh the complett repository (purge & clone instead of pull)
if [ "yes" == "${REBUILD}" ]; then
	if [ -d "${TMP_DIR}" ]; then
		rm -rf "${TMP_DIR}"
	fi

	# Fetch the source from remote endpoint
	$REPOSITORY_TYPE clone "${REPOSITORY}" "${TMP_DIR}"
fi



# If there where errors during {git,hg} clone, then ${TMP_DIR} does not exist
# and we can't continue
if [ -d "${TMP_DIR}" ]; then
	echo "Checkout seesm successful, continuing..."
else
	echo "Error while checkout, missing directory ${TMP_DIR}"
	exit 1
fi

# Ugly fix to work with the google repository which includes a trunk and
# branches directory instead of using mercurial branches
if [ -d "${TMP_DIR}/trunk" ]; then
	touch "${TMP_DIR}/LICENSE"
	touch "${TMP_DIR}/README.md"
	bash -c "cd ${TMP_DIR}; cp -r trunk/* ./"
fi



# Rebuild jar
echo ""
echo "Trying to apply patch. If it detects the patch doesn't match just skip"
echo ""
cp "build/makejar.diff" "${TMP_DIR}/makejar.diff"
bash -c "cd ${TMP_DIR}; patch -p1 < makejar.diff" 
bash -c "cd ${TMP_DIR}/infrastructure; ./bin/makejar.sh"
bash -c "cd ${TMP_DIR}/infrastructure/ace; bin/make normal etherpad"
cp "${TMP_DIR}/infrastructure/build/appjet.jar" "${TMP_DIR}/etherpad/appjet-eth-dev.jar"



# Testing the build
#bash -c "./build/test-build.sh"



# Increments the version & create the control file
REVISION="0"
if [ -f "${REVISION_FILE}" ]; then
	REVISION=`cat "${REVISION_FILE}"`
	REVISION=`expr $REVISION + 1`
fi
echo $REVISION > "${REVISION_FILE}"

# Patch the debain control file
cp "DEBIAN/control" "${TMP_DIR}/control.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/control.0" > "${TMP_DIR}/control.1"
sed "s/%REVISION%/${REVISION}/" "${TMP_DIR}/control.1" > "${TMP_DIR}/control.2"
cp "${TMP_DIR}/control.2" "${TMP_DIR}/control"

# Patch the install & deinstall script
cp "DEBIAN/prerm" "${TMP_DIR}/prerm"
cp "DEBIAN/postinst" "${TMP_DIR}/postinst.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/postinst.0" > "${TMP_DIR}/postinst.1"
cp "${TMP_DIR}/postinst.1" "${TMP_DIR}/postinst"

# Patch debconf templates
cp "DEBIAN/templates" "${TMP_DIR}/templates.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/templates.0" > "${TMP_DIR}/templates.1"
cp "${TMP_DIR}/templates.1" "${TMP_DIR}/templates"

# Patch init script
cp "etc/init.d/etherpad" "${TMP_DIR}/init.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/init.0" > "${TMP_DIR}/init.1"
cp "${TMP_DIR}/init.1" "${TMP_DIR}/init"

# Copy the config folder
cp -r "etc" "${TMP_DIR}/etc"



# Build the package enviroment (needed to build with dpgk-deb build)
if [ -d "${BUILD_DIR}" ]; then
	sudo rm -r "${BUILD_DIR}"
fi
mkdir "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/share/etherpad"
mkdir -p "${BUILD_DIR}/usr/share/doc/etherpad"
mkdir -p "${BUILD_DIR}/var/log/etherpad"



# Gather the required files
cp "${TMP_DIR}/control" "${BUILD_DIR}/DEBIAN/control"
cp "${TMP_DIR}/postinst" "${BUILD_DIR}/DEBIAN/postinst"
cp "${TMP_DIR}/prerm" "${BUILD_DIR}/DEBIAN/prerm"
cp "${TMP_DIR}/templates" "${BUILD_DIR}/DEBIAN/templates"
cp -r "${TMP_DIR}/etc" "${BUILD_DIR}/"
cp "${TMP_DIR}/init" "${BUILD_DIR}/etc/init.d/etherpad"
cp -r "${TMP_DIR}/etherpad" "${BUILD_DIR}/usr/share/etherpad"
cp "build/run.sh" "${BUILD_DIR}/usr/share/etherpad/etherpad/bin"
mkdir -p "${BUILD_DIR}/usr/share/etherpad/etherpad/data"
cp -r "${TMP_DIR}/infrastructure" "${BUILD_DIR}/usr/share/etherpad"
cp "${TMP_DIR}/COPYING" "${BUILD_DIR}/usr/share/doc/etherpad"
cp "${TMP_DIR}/LICENSE" "${BUILD_DIR}/usr/share/doc/etherpad"
cp "${TMP_DIR}/README.md" "${BUILD_DIR}/usr/share/doc/etherpad"



# Fix priviliges and build the package
sudo chown -R root:root "${BUILD_DIR}"
sudo chmod +x "${BUILD_DIR}/DEBIAN/postinst"
sudo chmod +x "${BUILD_DIR}/DEBIAN/prerm"
sudo chmod +x "${BUILD_DIR}/etc/init.d/etherpad"
sudo chmod -R 777 "${BUILD_DIR}/usr/share/etherpad/etherpad/data"

if [ -d "${PACKAGE_DIR}" ]; then
	rm -rf "${PACKAGE_DIR}"
fi
mkdir "${PACKAGE_DIR}"

dpkg-deb --build "${BUILD_DIR}" "${PACKAGE_DIR}"



# Transfer the package to local repository if environment variable 
PACKAGE=`bash -c "cd ${PACKAGE_DIR}; find . -name *.deb"`

if [ "yes" == "${DEPLOY_TO_LOCAL_REPOSITORY}" ]; then
	if [ -f "${PACKAGE_DIR}/${PACKAGE}" ]; then
		cp "${PACKAGE_DIR}/${PACKAGE}" "/var/www/packages"
		bash -c "cd /var/www/packages; reprepro -b . includedeb lenny ${PACKAGE}; rm ${PACKAGE}"
	else
		echo "No package in ${PACKAGE_DIR}"
	fi
fi

echo "Finished building package ${PACKAGE}"

