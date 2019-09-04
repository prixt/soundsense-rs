# SOUNDSENSE-RS : SoundSense, written in Rust. [![Build Status](https://travis-ci.org/prixt/soundsense-rs.svg?branch=master)](https://travis-ci.org/prixt/soundsense-rs)
My attempt at recreating [SoundSense](http://df.zweistein.cz/soundsense/), a sound-engine tool for [Dwarf Fortress](http://www.bay12games.com/dwarves/), using Rust.

## Why?
1. To see if I could do it.
2. Attempt to create a standalone application that doesn't require a Java runtime.
   * Ultimately, you should only need one executable, the soundpack folder, and DF.

## Current Features
* Plays sound reactive to what happens in DF.
* Adjust sound volumes realtime.
* Major sound parameters from the original (stereo balance, random balance, etc.)
* Custom ignore list, allowing user to customize which log patterns to ignore.
* Minimalist. (Though not necessarily by choice).

## Command line parameters
* __-l / --gamelog [GAMELOG_FILE] :__ preload the gamelog _(default: ".\gamelog.txt")_
* __-p / --soundpack [PACK_DIR] :__ preload the soundpack _(default: ".\soundpack")_
* __-i / --ignore [IGNORE_FILE] :__ preload the ignore list _(default: ".\ignore.txt")_
    * The ignore list is a simple text file, with each line being a regular expression. Any gamelog entries that match an expression in the ignore list will not be parsed.
    * Warning: the regex must follow the [regex crate syntax](https://docs.rs/regex/#syntax).


Example:

    soundsense-rs.exe -l "path/to/gamelog.txt" -p "path/to/soundpack/folder"
This will make soundsense-rs check if there is a file named "ignore.txt" in the binary directory, and will use that file to make the ignore list.

## Known problems
* 'battle/hit/punch/punch4.mp3', 'battle/hit/push/push5.mp3' cause 'DecoderError::UnrecognizedFormat'. Currently displays the error message and continues.
* not properly tested in Adventure mode & Arena mode.

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