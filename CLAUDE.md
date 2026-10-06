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
  State (mode, organ, preset, piano preset) is saved to `~/.organsynth.json`. Pedal stops get
  +4 dB (`PEDAL_GAIN_DB`). A watchdog exits the process if the audio output stops (e.g. JACK
  restarted) so pm2 restarts it.
- `src/devices.ts` — opens the piano (`/piano/i`) and pedalboard (`/teensy/i`) MIDI inputs, polls
  every second to survive unplugging/power cycles, and sends Local Control Off (CC 122 = 0) to the
  piano every second so its internal sound stays silent (the piano forgets it when powered off).
- `src/presets.ts` — the six numbered registrations (originally GrandOrgue General combinations
  1–6), described by stop family and footage and resolved against whichever organ is selected.
  Every pedal registration includes an 8' principal: the piano's small speakers barely play the
  16'/8' flute fundamentals.
- `scripts/local-control.ts` — `npm run local-control -- off|on` from any computer the piano is on.

Each device has its own input, filtered to its channel (piano 1, pedalboard 2), because
amidiminder connects every hardware port to both inputs. Organ mode sends the piano to
`organ.great` and the pedalboard to `organ.pedal`; piano mode sends the piano to the grand piano
(CC 64 = sustain) and ignores the pedalboard.

HTTP API: `GET /api/state`; `POST /api/mode {mode}`, `/api/organ {organ}`,
`/api/registration {registration}`, `/api/piano-preset {preset}`. Each returns the new state.
Volume is set on the piano; the Pi's DAC stays at 0 dB.

**stopmanager/** (React/Vite/Tailwind) — the phone UI: organ/piano switch, organ picker (‹ › and a
list), presets 1–6, piano presets. `npm run build` produces `dist/`,
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
- JACK runs as a system service (hw:DAC, 48 kHz); supersynth connects
  to it. amidiminder (`/etc/amidiminder.rules`) auto-connects every hardware and app MIDI port.
- JACK runs at 256 frames (`/etc/jackdrc`; 128 xruns on heavy registrations). pm2 runs under
  `pm2-patch.service` with a drop-in (`/etc/systemd/system/pm2-patch.service.d/realtime.conf`)
  giving it `LimitRTPRIO=95` and ordering it after `jack.service`, so the audio threads are real-time.
- The DAC (pcm512x) is at 0 dB with de-emphasis and auto-mute off (`alsactl store`); volume is
  set on the piano. Do not run `amixer` while audio plays: it causes xruns.
- Env overrides: `AUDIO_BACKEND`, `PORTS`, `STATE_FILE`, `THREADS` (4 in `ecosystem.config.js`).
