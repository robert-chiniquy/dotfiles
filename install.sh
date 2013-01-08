#! /bin/bash

ln -s `pwd`/.vimrc ~/.vimrc
ln -s `pwd`/.vim ~/.vim
ln -s `pwd`/.bash_login ~/.bash_login

git submodule update --init
