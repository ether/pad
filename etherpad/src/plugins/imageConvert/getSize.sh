#! /bin/bash

filename="$1"
page="$2"
mime="$(file -L -b -i "$filename" | sed -e "s+;.*++g")"

if [ "$mime" == "application/pdf" ]; then
  orderToOrientation () {
    read width
    read height
    if ((height > width)); then
      echo "portrait";
    else
      echo "landscape";
    fi
  }

  getPageOrientation () {
    {
      tmp="$(mktemp)"
      pdftoppm -png -f $2 -l $2 -scale-to 200 "$1" "$tmp"
      identify -format "%[fx:w]\n%[fx:h]" "$tmp"*.png
      rm "$tmp"*
    } | orderToOrientation
  }

  SHOULD_BE_ORIENTATION="$(getPageOrientation "$filename" "$page")"

  pdfinfo -f $page -l $page "$filename" | grep "Page.*size:" | sed -e "s+Page.*size: *\([0-9.]*\) x \([0-9.]*\) .*+\1\n\2+g" | {
    read width
    read height

    IS_ORIENTATION="$({ echo "$width"; echo "$height"; } | orderToOrientation)"

    if [ "$SHOULD_BE_ORIENTATION" == "$IS_ORIENTATION" ]; then
      echo "$width"; echo "$height";
    else
      echo "$height"; echo "$width"; 
    fi
  }
else
  identify -format "%[fx:w]\n%[fx:h]" "$filename[$page]"
fi
