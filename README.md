# SOUNDSENSE-RS : SoundSense, written in rust. [![Build Status](https://travis-ci.org/prixt/soundsense-rs.svg?branch=master)](https://travis-ci.org/prixt/soundsense-rs)
My attempt at recreating [SoundSense](http://df.zweistein.cz/soundsense/), a sound-engine tool for [Dwarf Fortress](http://www.bay12games.com/dwarves/), using Rust.

## Why?
1. To see if I could do it.
2. Attempt to create a standalone application that doesn't require a Java runtime.
   * Ultimately, you should only need one executable, the soundpack folder, and DF.

## Command line commands
* -l / --gamelog [GAMELOG_FILE] : preload the gamelog
* -p / --soundpack [PACK_DIR] : preload the soundpack

ex) soundsense-rs.exe -l "path/to/gamelog.txt" -p "path/to/soundpack/folder"

## Current Features
* Plays sound reactive to what happens in DF.
* Adjust sound volumes realtime.
* Minimalist. (not necessarily by choice).

## Known problems
* 'battle/hit/punch/punch4.mp3', 'battle/hit/push/push5.mp3' cause 'DecoderError::UnrecognizedFormat'. Currently displays the error message and continues.
* not properly tested in Adventure mode & Arena mode.
* weather and music loops sometimes don't restart playing after finishing.

## [License](./LICENSE)

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