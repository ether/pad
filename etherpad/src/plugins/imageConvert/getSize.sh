#! /bin/bash

filename="$1"
page="$2"
mime="$(file -L -b -i "$filename" | sed -e "s+;.*++g")"

if [ "$mime" == "application/pdf" ]; then
  pdfinfo -f $page -l $page "$filename" | grep "Page.*size:" | sed -e "s+Page.*size: *\([0-9.]*\) x \([0-9.]*\) .*+\1\n\2+g"
else
  identify -format "%[fx:w]\n%[fx:h]" "$filename[$page]"
fi
