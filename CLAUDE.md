# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OrganSynth turns a Raspberry Pi 5 into a pipe organ and piano for a Donner DEP-20 digital piano
plus a Teensy MIDI pedalboard. Sound comes from [supersynth](https://www.npmjs.com/package/@supersynth/core)
(`@supersynth/core` and the `@supersynth/organs` sample models). The Pi's DAC feeds the piano's speakers.

## Architecture

One Node process, run by pm2 (`ecosystem.config.js`, app `organsynth`):

**synth/** (TypeScript, run with tsx)
- `src/index.ts` — creates the supersynth `Synth` (JACK backend on Linux, overload guard on), a
  grand piano and the selected organ; routes MIDI; serves the HTTP API and the built web app on
  ports 8080 and 5173 (5173 is where the old Vite dev server ran, so existing bookmarks work).
  State (mode, organ, preset, piano preset) is saved to `~/.organsynth.json`.
- `src/devices.ts` — opens the piano (`/piano/i`) and pedalboard (`/teensy/i`) MIDI inputs, polls
  every second to survive unplugging/power cycles, and sends Local Control Off (CC 122 = 0) to the
  piano every second so its internal sound stays silent (the piano forgets it when powered off).
- `src/presets.ts` — the six numbered registrations (originally GrandOrgue General combinations
  1–6), described by stop family and footage and resolved against whichever organ is selected.

Each device has its own input, filtered to its channel (piano 1, pedalboard 2), because
amidiminder connects every hardware port to both inputs. Organ mode sends the piano to
`organ.great` and the pedalboard to `organ.pedal`; piano mode sends the piano to the grand piano
(CC 64 = sustain) and ignores the pedalboard.

HTTP API: `GET /api/state`; `POST /api/mode {mode}`, `/api/organ {organ}`,
`/api/registration {registration}`, `/api/piano-preset {preset}`, `/api/volume {volume}` (ALSA
`Digital` on card `DAC`, dB-mapped percent). Each returns the new state.

**stopmanager/** (React/Vite/Tailwind) — the phone UI: organ/piano switch, organ picker, presets
1–6, piano presets, HEADPHONE (12%) / SPEAKER (100%) output. `npm run build` produces `dist/`,
which the synth serves. `npm run dev` proxies `/api` to a synth on localhost:8080.

**util/** — older Python/shell helpers (GPIO note sensor, MIDI file player).

## Commands

```
cd synth && npm install && npm start      # requires JACK running on Linux
cd synth && npm run typecheck
cd stopmanager && npm install && npm run build
pm2 start ecosystem.config.js && pm2 save # on the Pi
```

## The Pi

- `patch@patchbox.local`, repo at `/home/patch/organsynth`, Patchbox OS (Bookworm, Pi 5, 4 GB).
- JACK runs as a system service (`/etc/jackdrc`: hw:DAC, 48 kHz, 128 frames); supersynth connects
  to it. amidiminder (`/etc/amidiminder.rules`) auto-connects every hardware and app MIDI port.
- Env overrides: `AUDIO_BACKEND`, `PORTS`, `STATE_FILE`, `VOLUME_CARD`, `VOLUME_CONTROL`.
