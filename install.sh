#! /bin/bash

[ ! -e .git ] || ( echo ":/" ; exit 1 ) 

[ -h ~/.vim ] && unlink .vim
[ -h ~/.vimrc ] && unlink .vimrc
[ -h ~/bash_login ] && unlink .bash_login

[ ! -e ~/.vimrc ] && ln -s `pwd`/.vimrc ~/.vimrc
[ ! -e ~/.vim ] && ln -s `pwd`/.vim ~/.vim
[ ! -e ~/.bash_login ] && ln -s `pwd`/.bash_login ~/.bash_login

git submodule update --init
