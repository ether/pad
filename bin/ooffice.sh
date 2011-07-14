#! /bin/bash

exec /usr/bin/soffice -headless -nofirststartwizard -accept="socket,host=localhost,port=8100;urp;StarOffice.Service" &
