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
  State (mode, organ, preset, piano preset, room per mode, pedal coupler) is saved to `~/.organsynth.json`.
  Master volume is per mode (`VOLUME`: organ 0.1, piano 0.5) so both play at the same level,
  times each organ's trim. Room: 'auto' (the instrument's own), 'off' or a supersynth reverb
  preset, per mode. A watchdog exits the process if the audio output stops (e.g. JACK
  restarted) so pm2 restarts it.
- `src/devices.ts` — opens the piano (`/piano/i`) and pedalboard (`/teensy/i`) MIDI inputs, polls
  every second to survive unplugging/power cycles, and sends Local Control Off (CC 122 = 0) to the
  piano every second so its internal sound stays silent (the piano forgets it when powered off).
- `src/presets.ts` — six registrations per organ, soft to loud (soft flute, flutes, principal,
  principal chorus, plenum, full organ), picked from each organ's own supersynth presets in
  `CHOICES`, with Title Case `LABELS`. `oneKeyboard()` couples every manual a preset uses to the
  great (the Donner is one keyboard), drops the presets' manual-to-pedal couplers (that's the
  player's Manual → Pedal button; `withPedalCoupled()` couples every manual with stops, since
  couplers don't chain) and adds an 8' pedal stop when the pedal has none (the piano's small
  speakers barely reproduce 16' fundamentals); soft (flute) registrations only ever get a flute.
  Organs without pedal stops (Green Positiv) are always coupled.
- `src/organ.ts` — builds an organ: pedal stops get +18 dB headroom (`PEDAL_HEADROOM_DB`), then
  each registration sets the pedal's expression from `src/balance.json`, and each organ gets a
  loudness trim from it.
- `scripts/balance.ts` — `npm run balance [-- <organ>…]` renders every registration offline and
  writes `src/balance.json`: pedal expression so a pedal note sits 6 dB under a manual triad
  (6 by ear on the Donner; uncoupled)
  (levels above 100 Hz, what the speakers reproduce), and per-organ trims (±6 dB) that match each
  organ's principal chorus to the median. Re-run it after changing presets or supersynth models.
- `scripts/local-control.ts` — `npm run local-control -- off|on` from any computer the piano is on.

Each device has its own input, filtered to its channel (piano 1, pedalboard 2), because
amidiminder connects every hardware port to both inputs. Organ mode sends the piano to
`organ.great` and the pedalboard to `organ.pedal`; piano mode sends the piano to the grand piano
(CC 64 = sustain; other controllers, e.g. CC 66 sostenuto and CC 67 soft, are forwarded) and
ignores the pedalboard. The Donner's three pedals are on/off (0/127) and are sent on channels 1, 2
and 3 at once, so the pedalboard input also sees them on channel 2 (harmless: only notes are used there).

HTTP API: `GET /api/state`; `POST /api/mode {mode}`, `/api/organ {organ}`,
`/api/registration {registration}`, `/api/piano-preset {preset}`, `/api/room {room}`, `/api/pedal-coupler {on}`. Each returns
the new state.
Volume is set on the piano; the Pi's DAC stays at 0 dB.

**stopmanager/** (React/Vite/Tailwind) — the phone UI: organ/piano switch, organ picker (‹ › and a
list), the six presets by name (equal-size buttons; `src/FitText.tsx` shrinks long names), piano
presets, Manual → Pedal coupler, room picker. `npm run build` produces `dist/`,
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
