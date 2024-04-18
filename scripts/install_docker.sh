#!/bin/bash
export DEBIAN_FRONTEND=noninteractive
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
rm get-docker.sh