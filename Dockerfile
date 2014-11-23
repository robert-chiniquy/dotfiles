FROM ubuntu:latest
MAINTAINER Robert Chiniquy "robert.chiniquy@gmail.com"

RUN apt-get -y update
RUN apt-get -y install git gcc gdb clang llvm strace lsof iotop curl atop mtr blktrace valgrind nmap sysstat htop vim silversearcher-ag golang python lua5.2 jp2a
# RUN curl https://raw.githubusercontent.com/creationix/nvm/v0.18.0/install.sh | bash
