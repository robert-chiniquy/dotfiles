#! /bin/bash

set -x
set -e

PROG=dot
#PROG=twopi
#PROG=neato

ls *.dot | while read fn ; do CPROG="`grep -oie '^\#PROG [a-z]*' $fn | sed -e 's/#PROG //g'`" ; [ -z "$CPROG" ] && export CPROG=$PROG ; $CPROG -Tpng -o`basename $fn .dot`.png $fn ; done

