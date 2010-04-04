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
# Baut etherpad aus den Quellen und erstellt ein Debian/Ubuntu Paket
#
# @param $1 (optional) Wenn rebuild, dann wird das Repository neu ausgecheckt
#     anstatt nur geupdated zu werden
# @param $REPOSITORY_URL URL von welcher das Repository geladen werden soll
# @param $REPOSITORY_NAME Name des Paketes welches gebaut werden soll
# @param $REPOSITORY_TYPE Element der Menge {hg, git}
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
# Folgende Parameter sind an das System auf welchem gebaupt wird anzupassen:
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
# Interne Konfiguration, muss nicht angepasst werden
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



# Einfaches Update ist nur ausreichend, wenn Repository bereits ausgecheckt
if [ "yes" != "${REBUILD}" ]; then
	if [ -d "${TMP_DIR}" ]; then
		bash -c "cd ${TMP_DIR}; ${REPOSITORY_TYPE} pull"
#		bash -c "cd ${TMP_DIR}; ${REPOSITORY_TYPE} checkout HEAD^1"
	else
		echo "Repository does not exist, will fetch"
		REBUILD="yes"
	fi
fi

# Repository komplett neu auschecken statt nur upzudaten
if [ "yes" == "${REBUILD}" ]; then
	if [ -d "${TMP_DIR}" ]; then
		rm -rf "${TMP_DIR}"
	fi

	# Source aus ,,Offizieller'' Quelle holen
	$REPOSITORY_TYPE clone "${REPOSITORY}" "${TMP_DIR}"
fi



# Checkout halbwegs ueberpruefen
if [ -d "${TMP_DIR}" ]; then
	echo "Checkout seesm successful, continuing..."
else
	echo "Error while checkout, missing directory ${TMP_DIR}"
	exit 1
fi

# Haesslicher Fix um mit dem Google Repository arbeiten zu koennen (trunk
# und branches in mercurial, wtf?)
if [ -d "${TMP_DIR}/trunk" ]; then
	touch "${TMP_DIR}/LICENSE"
	touch "${TMP_DIR}/README.md"
	bash -c "cd ${TMP_DIR}; cp -r trunk/* ./"
fi



# Jar neu bauen
echo ""
echo "Trying to apply patch. If it detects the patch doesn't match just skip"
echo ""
cp "build/makejar.diff" "${TMP_DIR}/makejar.diff"
bash -c "cd ${TMP_DIR}; patch -p1 < makejar.diff" 
bash -c "cd ${TMP_DIR}/infrastructure; ./bin/makejar.sh"
bash -c "cd ${TMP_DIR}/infrastructure/ace; bin/make normal etherpad"
cp "${TMP_DIR}/infrastructure/build/appjet.jar" "${TMP_DIR}/etherpad/appjet-eth-dev.jar"



# Testet die gebaute Version
#bash -c "./build/test-build.sh"



# Buildrevision erhoehen und Kontrolldatei bauen
REVISION="0"
if [ -f "${REVISION_FILE}" ]; then
	REVISION=`cat "${REVISION_FILE}"`
	REVISION=`expr $REVISION + 1`
fi
echo $REVISION > "${REVISION_FILE}"

# Debian-Control-File patchen
cp "DEBIAN/control" "${TMP_DIR}/control.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/control.0" > "${TMP_DIR}/control.1"
sed "s/%REVISION%/${REVISION}/" "${TMP_DIR}/control.1" > "${TMP_DIR}/control.2"
cp "${TMP_DIR}/control.2" "${TMP_DIR}/control"

# Installations- und Deinstallationsskript patchen
cp "DEBIAN/prerm" "${TMP_DIR}/prerm"
cp "DEBIAN/postinst" "${TMP_DIR}/postinst.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/postinst.0" > "${TMP_DIR}/postinst.1"
cp "${TMP_DIR}/postinst.1" "${TMP_DIR}/postinst"

# debconf Templates patchen
cp "DEBIAN/templates" "${TMP_DIR}/templates.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/templates.0" > "${TMP_DIR}/templates.1"
cp "${TMP_DIR}/templates.1" "${TMP_DIR}/templates"

# Init-Skript patchen
cp "etc/init.d/etherpad" "${TMP_DIR}/init.0"
sed "s/%BRANCH%/${BRANCH}/" "${TMP_DIR}/init.0" > "${TMP_DIR}/init.1"
cp "${TMP_DIR}/init.1" "${TMP_DIR}/init"

# Konfigurationsverzeichnis kopieren
cp -r "etc" "${TMP_DIR}/etc"



# Paketumgebung bauen
if [ -d "${BUILD_DIR}" ]; then
	sudo rm -r "${BUILD_DIR}"
fi
mkdir "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/share/etherpad"
mkdir -p "${BUILD_DIR}/usr/share/doc/etherpad"
mkdir -p "${BUILD_DIR}/var/log/etherpad"



# Benoetigte Dateien zusammenfuehren
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



# Eigentliches Paket bauen
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



# Paket in lokales Repository aufnehmen
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

