# organsynth

A lightweight, synthesized pipe organ emulator.

## Architecture

There are a few components to this repo:


* Synth- the rust organ emulator, which takes input from MIDI and produces sound via JACK.

* HTTPMIDI- a simple typescript server that sets up a virtual MIDI port, takes HTTP requests and converts them to MIDI events.

* StopManager- a simple react app that allows toggling organ stops and sending HTTP requests. This is designed to be run from a remote machine (ideally on the same network).