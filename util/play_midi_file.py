#!/usr/bin/env python3

import argparse
import time
import mido
import rtmidi

def all_notes_off(midiout, channel=0):
    """Send 'All Notes Off' for all 16 MIDI channels."""
    for ch in range(16):
        midiout.send_message([0xB0 + ch, 123, 0])  # 0xB0 = Control Change, 123 = All Notes Off

def send_cc(midiout, channel, cc_number, value=127):
    """Send a MIDI CC message on the given channel."""
    midiout.send_message([0xB0 + channel, cc_number, value])

def main():
    parser = argparse.ArgumentParser(description="Play a MIDI file through rtmidi")
    parser.add_argument("midi_file", help="Path to the MIDI file to play")
    parser.add_argument("--cc", action="append", metavar="CHANNEL:CC_NUMBER[:VALUE]",
                        help="Send MIDI CC before playback (repeatable). Format: CHANNEL:CC_NUMBER[:VALUE] (value defaults to 127)")
    args = parser.parse_args()

    # --- Load the MIDI file using Mido ---
    try:
        mid = mido.MidiFile(args.midi_file)
    except Exception as e:
        print(f"Failed to open MIDI file: {e}")
        raise SystemExit(1)

    # --- Create an rtmidi output object ---
    midiout = rtmidi.MidiOut()

    ports = midiout.get_ports()
    if ports:
        midiout.open_port(0)
        print(f"Opened MIDI port: {ports[0]}")
    else:
        midiout.open_virtual_port("pedalboard")
        print("No ports found, opened virtual port: pedalboard")

    # --- Send CC messages for preset selection ---
    if args.cc:
        for cc_spec in args.cc:
            try:
                parts = cc_spec.split(":")
                channel = int(parts[0])
                cc_number = int(parts[1])
                value = int(parts[2]) if len(parts) > 2 else 127
                send_cc(midiout, channel, cc_number, value)
                print(f"Sent CC {cc_number} value {value} on channel {channel}")
            except (ValueError, IndexError):
                print(f"Invalid --cc format: {cc_spec} (expected CHANNEL:CC_NUMBER[:VALUE])")
                raise SystemExit(1)
        time.sleep(0.1)

    print(f"Playing MIDI file: {args.midi_file}")
    start_time = time.time()

    try:
        # --- Real-time Playback ---
        for msg in mid.play():
            if not msg.is_meta:
                midi_bytes = msg.bytes()
                midiout.send_message(midi_bytes)

    except KeyboardInterrupt:
        print("\nPlayback interrupted. Sending all notes off...")
        all_notes_off(midiout)

    finally:
        # Ensure all notes are turned off when playback completes
        all_notes_off(midiout)
        midiout.close_port()

    duration = time.time() - start_time
    print(f"Finished playing in {duration:.2f} seconds.")

if __name__ == "__main__":
    main()
