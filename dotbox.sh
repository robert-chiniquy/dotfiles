#!/bin/bash

set -e

DOTBOX_DOCKER_IMAGE=dotfiles
DOTBOX_DOCKER_TAG=latest
DOTBOX_USER=root

machinename=$(echo "${USER}-${DOTBOX_DOCKER_IMAGE}-${DOTBOX_DOCKER_TAG}" | sed -r 's/[^a-zA-Z0-9_.-]/_/g')
machinepath="/var/lib/dotbox/${machinename}"
osrelease="${machinepath}/etc/os-release"

if [ ! -f ${osrelease} ] || systemctl is-failed -q ${machinename} ; then
  sudo mkdir -p "${machinepath}"
  sudo chown ${USER}: "${machinepath}"
  
  sudo mkdir -p ~/repo/
  sudo chown ${USER}: ~/repo/

  [ -e ~/repo/dotfiles ] || ( cd ~/repo && git clone https://github.com/robert-chiniquy/dotfiles )
  cd ~/repo/dotfiles && git pull

  docker build -t "${DOTBOX_DOCKER_IMAGE}:${DOTBOX_DOCKER_TAG}" .
  docker run --name=${machinename} "${DOTBOX_DOCKER_IMAGE}:${DOTBOX_DOCKER_TAG}" /bin/true
  docker export ${machinename} | sudo tar -x -C "${machinepath}" -f -
  docker rm ${machinename}

  sudo touch ${osrelease}
fi

sudo systemd-nspawn \
  -D "${machinepath}" \
  --share-system \
  --bind=/:/media/root  \
  --bind=/usr:/media/root/usr \
  --user="${DOTBOX_USER}" "$@"