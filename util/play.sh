#!/bin/bash
# Starts the synth and plays a MIDI file. Ctrl-C stops both.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SYNTH_DIR="$PROJECT_DIR/synth"

export DYLD_LIBRARY_PATH=/opt/homebrew/opt/jack/lib

# Map preset names to CC_NUMBER
preset_to_cc() {
    case "$1" in
        pedalboard_default)  echo "20" ;;
        manual_default)      echo "21" ;;
        mixture)             echo "23" ;;
        cornet)              echo "30" ;;
        manual_flute)        echo "24" ;;
        pedalboard_flute)    echo "25" ;;
        pedalboard_trumpet)  echo "26" ;;
        manual_trumpet)      echo "27" ;;
        plein_jeu)           echo "28" ;;
        pedalboard_reed)     echo "29" ;;
        *) echo ""; return 1 ;;
    esac
}

list_presets() {
    echo "Available presets:"
    echo "  pedalboard_default   (CC 20)"
    echo "  manual_default       (CC 21)"
    echo "  mixture              (CC 23)"
    echo "  cornet               (CC 30)"
    echo "  manual_flute         (CC 24)"
    echo "  pedalboard_flute     (CC 25)"
    echo "  pedalboard_trumpet   (CC 26)"
    echo "  manual_trumpet       (CC 27)"
    echo "  plein_jeu            (CC 28)"
    echo "  pedalboard_reed      (CC 29)"
}

# Default preset CC numbers to deactivate
DEFAULT_CCS="0 20 21"

# Parse arguments
CC_ARGS=""
MIDI_FILE=""
HAS_PRESETS=false

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
            cc_num=$(preset_to_cc "$1")
            if [ -z "$cc_num" ]; then
                echo "Error: unknown preset '$1'"
                echo ""
                list_presets
                exit 1
            fi
            HAS_PRESETS=true
            # Send this preset CC on all 16 channels
            for ch in $(seq 0 15); do
                CC_ARGS="$CC_ARGS --cc $ch:$cc_num"
            done
            shift
            ;;
        *)
            MIDI_FILE="$1"
            shift
            ;;
    esac
done

# Deactivate all default presets on all channels before activating explicit ones
if $HAS_PRESETS; then
    DEACTIVATE_ARGS=""
    for cc_num in $DEFAULT_CCS; do
        for ch in $(seq 0 15); do
            DEACTIVATE_ARGS="$DEACTIVATE_ARGS --cc $ch:$cc_num:0"
        done
    done
    CC_ARGS="$DEACTIVATE_ARGS$CC_ARGS"
fi

MIDI_FILE="${MIDI_FILE:-$SCRIPT_DIR/bwv552f.mid}"

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
cargo build --release --manifest-path "$SYNTH_DIR/Cargo.toml" || exit 1
cd "$SYNTH_DIR"
cargo run --release 2>&1 &
SYNTH_PID=$!

# Wait for synth to register with JACK
sleep 2

# Play MIDI file
python3 "$SCRIPT_DIR/play_midi_file.py" $CC_ARGS "$MIDI_FILE" &
MIDI_PID=$!

# Wait for MIDI playback to finish (or Ctrl-C)
wait $MIDI_PID 2>/dev/null
