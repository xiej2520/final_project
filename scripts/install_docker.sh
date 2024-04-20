#!/bin/bash
export DEBIAN_FRONTEND=noninteractive
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
rm get-docker.sh

sudo curl -SL https://raw.githubusercontent.com/junedkhatri31/docker-volume-snapshot/main/docker-volume-snapshot -o /usr/local/bin/docker-volume-snapshot
sudo chmod +x /usr/local/bin/docker-volume-snapshot
