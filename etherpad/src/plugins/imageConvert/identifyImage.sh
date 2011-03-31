#! /bin/bash

filename="$1"
page="$2"

pdfinfo -f $page -l $page "$filename" | grep "Page.*size:" | sed -e "s+Page.*size: *\([0-9.]*\) x \([0-9.]*\) .*+\1\n\2+g"
