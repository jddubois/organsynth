# organsynth

A pipe organ and piano for a Raspberry Pi, played from a digital piano and a MIDI pedalboard.

## Architecture

* Synth- a Node service built on [supersynth](https://www.npmjs.com/package/@supersynth/core). It
  plays the piano keyboard and pedalboard through a choice of sampled organs or a grand piano,
  keeps the digital piano's own sound switched off (MIDI Local Control Off), and serves the web app.

* StopManager- a react app for a phone on the same network: switch between organ and piano, pick
  an organ, choose one of six registrations and set the output level.

Open `http://patchbox.local:8080` (or `:5173`) on the phone.
