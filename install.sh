#! /bin/bash

set -e

[ `which locale-gen` ] && locale-gen en_US.UTF-8

[ -e .git ] || ( echo ":/" ; exit 1 ) 

[ -h ~/.vim ] && unlink -d ~/.vim
[ -h ~/bin ] && unlink -d ~/bin
[ -h ~/.vimrc ] && unlink ~/.vimrc
[ -h ~/bash_login ] && unlink ~/.bash_login

[ ! -e ~/.vimrc ] && ln -s `pwd`/.vimrc ~/.vimrc
[ ! -e ~/.vim ] && ln -s `pwd`/.vim ~/.vim
[ ! -e ~/.bash_login ] && ln -s `pwd`/.bash_login ~/.bash_login
[ ! -e ~/bin ] && ln -s `pwd`/bin ~/bin

git submodule update --init
