#!/bin/sh

find ../../src -name "*.js" -exec ./scan_file_for_extra_commas.sh {} \;
