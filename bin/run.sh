#! /bin/bash


source exports.sh
cd etherpad
# the argument here is the maximum amount of RAM to allocate
exec bin/run-local.sh 256M
