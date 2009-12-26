#!/bin/bash

base=`dirname $0`
ebase=$base/etherpad
config=$base/.config
config_version=7
env=$base/env.sh


if [[ $1 == clean ]]; then
  rm -f .config
fi

if [[ -f $config ]]; then
  . $config
  if [[ $saved_config_version != $config_version ]]; then
    echo "Your saved config is too old.  You need to setup again."
    rm -f $config && exec $0
  fi
else
  echo -n "Domain for your etherpad install (e.g. etherpad.com): "
  read domain

  echo -n "IP (or hostname) and port to bind to (e.g. 127.0.0.1:9000): "
  read listen_at

  echo -n "Password to use for admin access (stored as plain text): "
  read admin_pass
fi

echo "Using domain $domain binding to $listen_at with admin pass $admin_pass"

REPLACE_ME="$ebase/src/static/crossdomain.xml.in
            $ebase/etc/etherpad.localdev-default.properties.in"

for in_file in $REPLACE_ME; do
  in_file_out=${in_file%.in}
  echo "Applying settings to $in_file_out"
  sed "s/FROM_DOMAIN/$domain/g;
       s/LISTEN_AT/$listen_at/g;
       s/EMAIL_FROM_ADDR/etherpad@$domain/g;
       s/ADMIN_PASS/$admin_pass/g;" $in_file > $in_file_out
done

echo "Saving your config settings for later."
echo "
domain=$domain
listen_at=$listen_at
admin_pass=$admin_pass
saved_config_version=$config_version
" > $base/.config

if [[ ! -f $env ]]; then
  echo "You need to copy $base/env.sh.template to $env and set proper values."
  exit 2
else
  . $env
fi

pushd $base/etherpad >& /dev/null
./bin/rebuildjar.sh
popd >& /dev/null

echo "Now use $base/run.sh to get things started."

