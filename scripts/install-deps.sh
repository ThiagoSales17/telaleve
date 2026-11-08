#!/bin/sh
set -eu
sudo apt-get update
sudo apt-get install -y build-essential pkg-config curl npm libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev libxdo-dev libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-nice
