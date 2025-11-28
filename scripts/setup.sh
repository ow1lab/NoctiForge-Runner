#!/bin/sh
set -euo pipefail

echo "Creating /var/lib/noctiforge directory with proper permissions..."
sudo mkdir -pv /var/lib/noctiforge/{registry,controlplane,native_worker/{pkgs,run}}
sudo chown -R "$(id -u):$(id -g)" /var/lib/noctiforge
