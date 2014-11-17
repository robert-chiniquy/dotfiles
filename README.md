### local dotfiles
`git clone https://github.com/robert-chiniquy/dotfiles.git && cd dotfiles && ./install.sh`
### remote dotbox
```
git clone https://github.com/robert-chiniquy/dotfiles.git
cd dotfiles
. update.sh
fab dotbox:host=core@<IP of coreos node>
```
### test change against vagrant
`vagrant destroy -f && vagrant up`