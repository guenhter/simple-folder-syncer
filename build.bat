@echo off

echo "Building..."
cargo build --release

echo "Packaging..."
wix build .\package.wxs -o SimpleFolderSyncer.msi
