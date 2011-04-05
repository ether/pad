#! /bin/bash

exec 2>>/tmp/xxx >&2
set -x
# pdfconvert INPUTFILE OUTPUTFILE PAGE DPI_X DPI_Y BB_X1 BB_Y1 BB_X2 BB_Y2

echo convertImage.sh "$@" > /tmp/xxx

tmp1=$(mktemp)
tmp2=$(mktemp)

input="$1"
output="$2"
page="$3"
resx="$4"
resy="$5"
x1="$6"
y1="$7"
x2="$8"
y2="$9"

pdf2ps -dFirstPage=$page -dLastPage=$page "$input" $tmp1
ps2epsi $tmp1 $tmp2
sed -ie "s/%%BoundingBox: [0-9]* [0-9]* [0-9]* [0-9]*/%%BoundingBox: $x1 $y1 $x2 $y2/" $tmp2
sed -ie "s/%%HiResBoundingBox: [0-9.]* [0-9.]* [0-9.]* [0-9.]*/%%BoundingBox: ${x1}.000000 ${y1}.000000 ${x2}.000000 ${y2}.000000/" $tmp2
gs -sDEVICE=pnggray -o "$output" -r${resx}x${resy} -dEPSCrop $tmp2

