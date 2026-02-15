#!/bin/bash
# Starts the synth and plays a MIDI file. Ctrl-C stops both.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MIDI_FILE="${1:-$SCRIPT_DIR/bwv588.mid}"
SYNTH_DIR="$PROJECT_DIR/synth"

export DYLD_LIBRARY_PATH=/opt/homebrew/opt/jack/lib

cleanup() {
    echo "Shutting down..."
    kill $MIDI_PID 2>/dev/null
    kill $SYNTH_PID 2>/dev/null
    wait $MIDI_PID 2>/dev/null
    wait $SYNTH_PID 2>/dev/null
    echo "Done."
}
trap cleanup EXIT INT TERM

# Build and start synth (must run from synth/ so Config.toml resolves)
cargo build --manifest-path "$SYNTH_DIR/Cargo.toml" || exit 1
cd "$SYNTH_DIR"
cargo run 2>&1 &
SYNTH_PID=$!

# Wait for synth to register with JACK
sleep 2

# Play MIDI file
python3 "$SCRIPT_DIR/play_midi_file.py" "$MIDI_FILE" &
MIDI_PID=$!

# Wait for MIDI playback to finish (or Ctrl-C)
wait $MIDI_PID 2>/dev/null
