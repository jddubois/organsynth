#!/bin/bash
# Starts the synth and plays a MIDI file. Ctrl-C stops both.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SYNTH_DIR="$PROJECT_DIR/synth"

export DYLD_LIBRARY_PATH=/opt/homebrew/opt/jack/lib

# Map preset names to CHANNEL:CC_NUMBER
preset_to_cc() {
    case "$1" in
        pedalboard_default)  echo "1:20" ;;
        manual_default)      echo "0:21" ;;
        organo_pleno)        echo "0:22" ;;
        mixture)             echo "0:23" ;;
        manual_flute)        echo "0:24" ;;
        pedalboard_flute)    echo "1:25" ;;
        pedalboard_trumpet)  echo "1:26" ;;
        manual_trumpet)      echo "0:27" ;;
        plein_jeu)           echo "0:28" ;;
        pedalboard_reed)     echo "1:29" ;;
        *) echo ""; return 1 ;;
    esac
}

list_presets() {
    echo "Available presets:"
    echo "  pedalboard_default   (channel 1, CC 20)"
    echo "  manual_default       (channel 0, CC 21)"
    echo "  organo_pleno         (channel 0, CC 22)"
    echo "  mixture              (channel 0, CC 23)"
    echo "  manual_flute         (channel 0, CC 24)"
    echo "  pedalboard_flute     (channel 1, CC 25)"
    echo "  pedalboard_trumpet   (channel 1, CC 26)"
    echo "  manual_trumpet       (channel 0, CC 27)"
    echo "  plein_jeu            (channel 0, CC 28)"
    echo "  pedalboard_reed      (channel 1, CC 29)"
}

# Parse arguments
CC_ARGS=""
MIDI_FILE=""

while [ $# -gt 0 ]; do
    case "$1" in
        --list-presets)
            list_presets
            exit 0
            ;;
        --preset)
            shift
            if [ -z "$1" ]; then
                echo "Error: --preset requires a name"
                exit 1
            fi
            cc=$(preset_to_cc "$1")
            if [ -z "$cc" ]; then
                echo "Error: unknown preset '$1'"
                echo ""
                list_presets
                exit 1
            fi
            CC_ARGS="$CC_ARGS --cc $cc"
            shift
            ;;
        *)
            MIDI_FILE="$1"
            shift
            ;;
    esac
done

MIDI_FILE="${MIDI_FILE:-$SCRIPT_DIR/bwv588.mid}"

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
python3 "$SCRIPT_DIR/play_midi_file.py" $CC_ARGS "$MIDI_FILE" &
MIDI_PID=$!

# Wait for MIDI playback to finish (or Ctrl-C)
wait $MIDI_PID 2>/dev/null
