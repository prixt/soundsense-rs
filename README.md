# SOUNDSENSE-RS : SoundSense, written in rust.
My attempt at recreating [SoundSense](http://df.zweistein.cz/soundsense/), a sound-engine tool for [Dwarf Fortress](http://www.bay12games.com/dwarves/), using Rust.

## Why?
1. To see if I could do it.
2. Attempt to create a standalone application that doesn't require a Java runtime.
   * Ultimately, you should only need one executable, the soundpack folder, and DF.

## Current Features
* Plays sound reactive to what happens in DF.
* Minimalist. (not necessarily by choice).

## Dependencies
* [notify](https://github.com/passcod/notify)
* [rodio](https://github.com/tomaka/rodio)
* [regex](https://github.com/rust-lang/regex)
* [rand](https://github.com/rust-random/rand)
* [gtk-rs](https://gtk-rs.org/)

## Known problems
* some regex expressions in the soundpacks don't parse using the 'regex' crate.
* 'battle/hit/punch/punch4.mp3', 'battle/hit/push/push5.mp3' cause 'DecoderError::UnrecognizedFormat'. Currently displays the error message and continues.
* many sound and soundfile attributes are currently ignored. (etc. stereo balance, delay, timeout, weighted random balance)
* having too many log entries at the same time causes the sound thread to stutter, and doesn't recover.
* not properly tested in Adventure mode & Arena mode.