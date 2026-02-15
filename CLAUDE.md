# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

OrganSynth is a lightweight, synthesized pipe organ emulator. It runs on a Raspberry Pi (or similar Linux device) with JACK audio and MIDI hardware.

## Architecture

Three components work together:

**synth/** (Rust) — The core organ synthesizer. Receives MIDI via JACK, generates audio output via JACK. This is the real-time audio engine.
- `main.rs` sets up the JACK client, registers audio/MIDI ports, and starts the MIDI listener
- `jack_handler.rs` implements JACK's `ProcessHandler` — reads MIDI events and writes audio samples each process cycle
- `synth/synth.rs` (`Synth`) is the outer synth that owns a channel-based MIDI sender and spawns a worker thread for MIDI processing
- `synth/thingy.rs` (`InternalSynth`) is the per-channel synth that manages notes, stops, and audio filters (low-pass + reverb). Named `thingy` as a placeholder — needs renaming
- `synth/note.rs` — a Note holds multiple Oscillators (one per active stop)
- `synth/oscillator.rs` — generates waveform samples with ADSR envelope and A-weighting loudness compensation
- `synth/waveform.rs` — waveform generators: sine, square, sawtooth, triangle (frequency-dependent sine/triangle blend for organ tone), trumpet (saturated sawtooth)
- `synth/stop.rs` — a Stop defines a waveform + frequency ratio + amplitude ratio
- `synth/config.rs` — resolves named/inline stops and presets from config into runtime Stop structs
- `midi/` — MIDI message parsing and a listener that auto-connects new JACK MIDI output ports
- `config/` — deserializes `Config.toml` (JACK settings, stops, presets, preset defaults)

**httpmidi/** (TypeScript/Express) — HTTP-to-MIDI bridge server on port 8080. Creates a virtual MIDI port ("httpmidi") and exposes:
- `POST /midi` — send raw MIDI messages
- `GET /config` — returns parsed synth config from `Config.toml`
- `GET/POST /volume` — ALSA volume control (hardware-specific)

**stopmanager/** (React/Vite/TypeScript) — Web UI for selecting organ presets and adjusting volume. Sends MIDI CC messages to httpmidi. Designed for use from a remote device on the same network.

**util/** (Python) — Hardware utilities:
- `note.py` — reads GPIO sensor input on a Raspberry Pi and sends MIDI note on/off via rtmidi
- `play_midi_file.py` — plays a MIDI file through rtmidi

## Data Flow

MIDI keyboard → JACK → synth (Rust) → JACK audio out
StopManager (React) → HTTP → httpmidi (Express) → virtual MIDI port → JACK → synth
GPIO sensor (Python) → virtual MIDI port → JACK → synth

## Configuration

`Config.toml` (project root) is the single source of truth for:
- JACK port names and client configuration
- Organ stop definitions (waveform, frequency ratio, amplitude ratio)
- Presets (collections of stops mapped to MIDI CC identifiers)
- Preset defaults (which preset each MIDI channel starts with)

Presets can reference stops by name (from `[synth.stops]`) or define inline stops. Each preset has a `midi_identifier` (CC number) and `channels` (which MIDI channels it applies to).

## Build & Run Commands

**Synth (Rust):**
```
cd synth && cargo build
cd synth && cargo run        # requires JACK server running
```

**httpmidi (TypeScript):**
```
cd httpmidi && npm install
cd httpmidi && npm start     # runs on port 8080
```

**StopManager (React):**
```
cd stopmanager && npm install
cd stopmanager && npm start  # vite dev server with --host
```

**All services via PM2:**
```
pm2 start ecosystem.config.js
```
Note: `ecosystem.config.js` hardcodes paths to `/home/patch/organsynth/` (the Pi deployment path).

## Key Dependencies

- Rust synth requires JACK audio (`jack` crate) and `pkg-config` with JACK dev headers installed
- httpmidi requires the `midi` npm package (native MIDI bindings) and `alsa-volume` (ALSA bindings — Linux only)
- StopManager hardcodes the httpmidi server IP (`192.168.1.21:8080`) in `App.tsx` and `config.tsx`
- Python utilities require `rtmidi`, `mido`, and `gpiod`

## Important Notes

- The synth reads `Config.toml` from `../Config.toml` relative to the `synth/` directory
- MIDI channels are 1-indexed in config but 0-indexed in the protocol (the code handles this)
- The `Triangle` waveform in `waveform.rs` actually generates a frequency-dependent sine/triangle blend (called `generate_organ_sample`), not a pure triangle wave
- Audio output is scaled by `0.05` in `InternalSynth::next_sample()` as a master volume control
