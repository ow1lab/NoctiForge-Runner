#!/bin/sh
set -euo pipefail

echo "Creating /var/lib/noctiforge directory with proper permissions..."
sudo mkdir -pv /var/lib/noctiforge/{registry,controlplane}
sudo chown -R "$(id -u):$(id -g)" /var/lib/noctiforge
