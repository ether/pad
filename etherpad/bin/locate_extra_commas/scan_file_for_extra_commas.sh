#!/bin/sh

echo $1

# OS X ... download, unpack and move rhino1_7R2 to /usr/local and then uncomment the following line
# alias rhino='java -jar /usr/local/rhino1_7R2/js.jar'

rhino ./jslint.js $1 | fgrep -e 'Extra comma' --color=always
