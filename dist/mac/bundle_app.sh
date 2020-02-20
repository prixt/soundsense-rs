#!/usr/bin/env bash
cp "target/release/soundsense-rs" "dist/mac/soundsense-rs.app/Content/MacOS/soundsense-rs";
chmod +x "dist/mac/soundsense-rs.app/Content/MacOS/soundsense-rs";
mv "dist/mac/soundsense-rs.app/" "./soundsense-rs.app/";