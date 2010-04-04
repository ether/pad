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





bash -c "`dirname $0`/run-local.sh" > "/var/log/etherpad/log" 2> "/var/log/etherpad/error"

