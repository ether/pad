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
# Must be called by build.sh
#####



# Truncate database to test start etherpad
echo "I'm going to lunch MySQL now, you maybe have to enter your root@localhost password"
sudo /etc/init.d/mysql start

DATABASE_NAME="etherpad"
DATABASE_USER="etherpad"
DROP_DATABASE="DROP DATABASE ${DATABASE_NAME}"
CREATE_DATABASE="CREATE DATABASE ${DATABASE_NAME}"
DROP_USER="DROP USER '${DATABASE_USER}'@'localhost'"
CREATE_USER="GRANT ALL PRIVILEGES ON ${DATABASE_NAME}.* TO '${DATABASE_USER}'@'localhost' identified by 'password';"
echo "${DROP_DATABASE}; ${CREATE_DATABASE}; ${DROP_USER}; ${CREATE_USER}" | mysql -u root -p | grep etherpad



# Start etherpad and wait for the HTTP-Server to be available
echo "I'm going to lunch etherpad in a clean environment, try if it works on http://localhost:9000/"
echo "Kill with ^C to build debian package (waiting 10 seconds)"
sleep 10
bash -c "cd ${TMP_DIR}/etherpad; ./bin/run-local.sh"

