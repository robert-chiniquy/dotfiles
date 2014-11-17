
from fabric.api import env, run
from fabric.operations import put, sudo, local
from fabric.contrib.files import exists

env.user = "core"

def dotbox():
  sudo("mkdir -p /opt/bin /var/lib/dotbox")
  put("dotbox.sh", '/opt/bin/dotbox', use_sudo=True) 
  sudo("chmod +x /opt/bin/dotbox")
  run("dotbox bash -i -c 'echo -e \"\033[0;31m <3 \033[0m \"'")
  sudo("sudo chown -R core:core /var/lib/dotbox/dotfiles-latest/root")
  if not exists("/var/lib/dotbox/dotfiles-latest/root/dotfiles"):
    run("ssh-keyscan github.com >> /home/core/.ssh/known_hosts")
    local("eval `ssh-agent`; ssh -A "+ env.host_string +" git clone git@github.com:robert-chiniquy/dotfiles.git /var/lib/dotbox/dotfiles-latest/root/dotfiles")  
  else:
    local("eval `ssh-agent`; ssh -A "+ env.host_string +" cd /var/lib/dotbox/dotfiles-latest/root/dotfiles && git pull")  
  run("dotbox bash -i -c 'cd /root/dotfiles ; ./install.sh'")
