#!/bin/bash

################################################################################
#
# Copyright (c) 2010 penSec.IT UG (haftungsbeschränkt)
#        http://www.pensec.it
#        mail@pensec.it
# 
# Diese Software wird ohne ausdrückliche oder implizierte Garantie
# bereitgestellt. Auf keinen Fall können die Autoren für irgendwelche Schäden,
# die durch die Benutzung dieser Software entstehen, haftbar gemacht werden.
# 
# Es ist dem Auftraggeber gestattet diese Software für jeden Zweck, inklusive
# kommerzieller Anwendungen, zu benutzten und zu verändern aber nicht
# weiterzuverbreiten, solange folgende Bedingungen erfüllt sind:
# 
#     1. Die Herkunft dieser Software darf nicht falsch dargestellt werden; Sie
#        dürfen nicht angeben, dass Sie die ursprüngliche Software geschrieben
#        haben. Wenn Sie diese Software in einem Produkt benutzten, würde eine
#        Erwähnung geschätzt werden, sie ist aber nicht erforderlich.
#     2. Veränderte Quelltextversionen müssen deutlich als solche
#        gekennzeichnet werden und dürfen nicht als die Originalsoftware
#        dargestellt werden.
#     3. Diese Notiz darf in den Quelltexten nicht verändert oder gelöscht
#        werden.
#
################################################################################





#####
# Muss von rebuild.sh aufgerufen werden
#####



# Datenbank leeren damit etherpad testweiste gestartet werden kann
echo "I'm going to lunch MySQL now, you maybe have to enter your root@localhost password"
sudo /etc/init.d/mysql start

DATABASE_NAME="etherpad"
DATABASE_USER="etherpad"
DROP_DATABASE="DROP DATABASE ${DATABASE_NAME}"
CREATE_DATABASE="CREATE DATABASE ${DATABASE_NAME}"
DROP_USER="DROP USER '${DATABASE_USER}'@'localhost'"
CREATE_USER="GRANT ALL PRIVILEGES ON ${DATABASE_NAME}.* TO '${DATABASE_USER}'@'localhost' identified by 'password';"
echo "${DROP_DATABASE}; ${CREATE_DATABASE}; ${DROP_USER}; ${CREATE_USER}" | mysql -u root -p | grep etherpad



# Etherpad starten und warten bis HTTP-Server laeuft
echo "I'm going to lunch etherpad in a clean environment, try if it works on http://localhost:9000/"
echo "Kill with ^C to build debian package (waiting 10 seconds)"
sleep 10
bash -c "cd ${TMP_DIR}/etherpad; ./bin/run-local.sh"





