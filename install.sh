#! /bin/bash

set -e

UNLINK_DIR_FLAG='-d'

# ubuntu unlink doesn't take -d
[ -e /etc/os-release ] && unset UNLINK_DIR_FLAG

[ `which locale-gen` ] && locale-gen en_US.UTF-8

[ -e .git ] || ( echo ":/" ; exit 1 )

[ -h ~/.vim ] && unlink $UNLINK_DIR_FLAG ~/.vim
[ -h ~/bin ] && unlink $UNLINK_DIR_FLAG ~/bin
[ -h ~/.vimrc ] && unlink ~/.vimrc
[ -h ~/.bash_login ] && unlink ~/.bash_login
[ -h ~/.inputrc ] && unlink ~/.inputrc

[ ! -e ~/.vim ] && ln -s `pwd`/.vim ~/.vim
[ ! -e ~/bin ] && ln -s `pwd`/bin ~/bin
[ ! -e ~/.vimrc ] && ln -s `pwd`/.vimrc ~/.vimrc
[ ! -e ~/.bash_login ] && ln -s `pwd`/.bash_login ~/.bash_login
[ ! -e ~/.inputrc ] && ln -s `pwd`/.inputrc ~/.inputrc

git submodule update --init
