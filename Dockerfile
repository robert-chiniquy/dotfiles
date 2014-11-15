FROM ubuntu:latest
MAINTAINER Robert Chiniquy "robert.chiniquy@gmail.com"

RUN apt-get -y update
RUN apt-get -y install git gcc gdb strace lsof iotop curl atop mtr blktrace nmap sysstat htop vim silversearcher-ag golang python lua5.2
# RUN curl https://raw.githubusercontent.com/creationix/nvm/v0.18.0/install.sh | bash
