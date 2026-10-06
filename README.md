# organsynth

A pipe organ and piano on a Raspberry Pi 5, played from a Donner DEP-20 digital piano and a MIDI
pedalboard, with the sound coming out of the piano's own speakers.

Sound comes from [supersynth](https://www.npmjs.com/package/@supersynth/core): 17 sampled church
organs (`@supersynth/organs`) and a grand piano. The digital piano's internal sound is switched off
(MIDI Local Control Off, re-sent every second), so its keys only play the Pi.

## Playing

Open **http://patchbox.local:5173** (or `:8080`) on a phone on the same network.

- **Organ / Piano** switches what the keyboard plays. In piano mode the sustain pedal works and
  the pedalboard is silent.
- **Organ**: step through the organs with ‹ ›, or pick one from the list.
- **Presets**: six per organ, soft to loud: Soft Flute, Flutes, Principal, Principal Chorus,
  Plenum, Full Organ (some organs use their own names, e.g. Fonds, Plein Jeu, Grand Chœur,
  Ripieno). They are the organ's own registrations, coupled so the one keyboard plays them, with
  the pedal level measured and set to sit under the manual.
- **Manual → Pedal** couples the manual to the pedal (the pedalboard also plays the manual
  stops); off by default. The Green Positiv has no pedal stops, so it's always coupled.
- **Room**: Default (the organ's own church, or the piano's hall), None, or one of supersynth's rooms; the piano has
  its own setting.
- **Piano presets**: default, bright, mellow, felt, concert, studio, honky-tonk, long sustain.
- **Volume** is the piano's volume knob; the Pi always plays at full level.

Your choices are remembered across restarts.

## How it works

```
Donner (MIDI ch 1) ─┐                                   ┌─ JACK ─ DAC ─ piano aux in ─ speakers
                    ├─ synth/ (Node, supersynth engine) ┤
Teensy pedalboard   ┘   ↑ Local Control Off → Donner     └─ HTTP :8080/:5173 ─ stopmanager/ (phone)
(MIDI ch 2)
```

- **synth/** — the service: supersynth with a grand piano and the selected organ, MIDI input from
  both devices (reconnecting after unplugging or power cycles), Local Control Off to the piano,
  and the HTTP API plus the built web app. `src/presets.ts` picks each organ's six presets;
  `npm run balance` measures them and stores the pedal levels and organ trims in `src/balance.json`.
- **stopmanager/** — the React phone UI, built to `dist/` and served by the synth.
- **util/** — older helpers (GPIO note sensor, MIDI file player).

## Running and deploying

On the Pi (`patch@patchbox.local`, repo in `~/organsynth`), pm2 runs the service at boot:

```bash
cd ~/organsynth && git pull
cd synth && npm ci && cd ../stopmanager && npm ci && npm run build && cd ..
pm2 restart organsynth        # first time: pm2 start ecosystem.config.js && pm2 save
pm2 logs organsynth
```

Locally: `cd synth && npm install && npm start` (any audio device; on Linux it expects JACK), and
`cd stopmanager && npm run dev` for the UI with hot reload. To silence or restore the piano from a
computer it's plugged into: `cd synth && npm run local-control -- off` (or `on`).

### Pi setup

These are already in place on the Pi; redo them on a fresh install:

- **No GrandOrgue**: its autostart line in `~/.config/lxsession/LXDE-pi/autostart` is commented out.
- **JACK** at 256 frames: `/etc/jackdrc` runs `jackd … -d alsa -d hw:DAC -r 48000 -p 256 -n 2`.
  128 frames crackles on heavy registrations.
- **Real-time audio for pm2**: `/etc/systemd/system/pm2-patch.service.d/realtime.conf` sets
  `LimitRTPRIO=95` and `LimitMEMLOCK=infinity` (systemd services don't get the audio group's
  limits), and starts pm2 after JACK.
- **DAC** (pcm512x): Digital 0 dB, Deemphasis off, Auto Mute off, saved with `sudo alsactl store`.
  Don't run `amixer` while audio plays: it causes dropouts.
- `THREADS=4` in `ecosystem.config.js`: all four cores render audio.
