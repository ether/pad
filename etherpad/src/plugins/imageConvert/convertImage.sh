#! /bin/bash

exec 2>/tmp/xxx >&2
set -x
# pdfconvert INPUTFILE OUTPUTFILE PAGE DPI_X DPI_Y BB_X BB_Y BB_W BB_H

echo convertImage.sh "$@"

tmp=$(mktemp)

input="$1"
output="$2"
page="$3"
resx="$4"
resy="$5"
x="$6"
y="$7"
w="$8"
h="$9"

pdftoppm -png -f $page -l $page -rx $resx -ry $resy -x $x -y $y -W $w -H $h "$input" "$tmp"
mv "$tmp"*.png "$output"
