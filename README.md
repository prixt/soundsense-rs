# <p align="center">SOUNDSENSE-RS</br><img src="./icons/icon.ico" width="128px" height="128px"></img></br>SoundSense, written in Rust.</br>![Release](https://github.com/prixt/soundsense-rs/workflows/Release/badge.svg?branch=release) ![Build](https://github.com/prixt/soundsense-rs/workflows/Build/badge.svg)</p>
My attempt at recreating [SoundSense](http://df.zweistein.cz/soundsense/), a sound-engine utility for [Dwarf Fortress](http://www.bay12games.com/dwarves/), using Rust.

<p align='center'>
    <img src="./screenshots/windows-screenshot.png" title='Windows screenshot' width='40%'>
    <img src="./screenshots/macos-screenshot.png" title='MacOs screenshot by jecowa' width='40%'>
</p>

## Why?
1. To see if I could do it.
2. Attempt to create a standalone application that doesn't require bloat (Java VM, C# VM, gtk, etc...).
   * Ultimately, you should only need one executable, the soundpack folder, and DF.
   * Recommended soundpack fork: https://github.com/jecowa/soundsensepack

## Current Features
* Plays sound reactive to what happens in DF.
* Can adjust sound volumes realtime, by channel.
* Most sound parameters from the original (stereo balance, random balance, etc.)
* Custom ignore list, allowing user to customize which log patterns to ignore.
* Simple and Clean GUI.
* Low memory requirement.

## Command line parameters
* __-l / --gamelog [GAMELOG_FILE] :__ preload the gamelog _(default: ".\gamelog.txt")_
* __-p / --soundpack [PACK_DIR] :__ preload the soundpack _(default: ".\soundpack")_
* __-i / --ignore [IGNORE_FILE] :__ preload the ignore list _(default: ".\ignore.txt")_
    * The ignore list is a simple text file, with each line being a regular expression. Any gamelog entries that match an expression in the ignore list will not be parsed.
* __--no-config :__ Don't read config files on start. Will use the given paths, or soundsense-rs defaults.

Example:

    soundsense-rs.exe -l "path/to/gamelog.txt" -p "path/to/soundpack/folder"
This will make soundsense-rs check if there is a file named "ignore.txt" in the current working directory, and will use that file to make the ignore list.

## Ignore List
Each line in the ignore list file is considered an regex pattern. If a gamelog message matches any of the patterns, that message is ignored.

Example:

    (.+) cancels (.+): (.*)(Water|water)(.*)\.
This pattern will make soundsense-rs ignore any cancallations related to water.

The regex pattern uses the [regex crate](https://docs.rs/regex/) syntax.

## [MIT License](./LICENSE)

Copyright (c) prixt

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## CSS Resources
* [W3.CSS](https://www.w3schools.com/w3css/)
* [range.css](http://danielstern.ca/range.css/#/)