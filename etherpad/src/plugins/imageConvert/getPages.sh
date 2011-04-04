#! /bin/bash

filename="$1"
mime="$(file -L -b -i "$filename" | sed -e "s+;.*++g")"

if [ "$mime" == "application/pdf" ]; then
  pdfinfo "$filename" | grep "Pages:" | sed -e "s+Pages: *++g"
else
  identify -format "%[scenes]" "$filename"
fi
