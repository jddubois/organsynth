#!/usr/bin/env python3

import sys
import time
import mido
import rtmidi

def all_notes_off(midiout, channel=0):
    """Send 'All Notes Off' for all 16 MIDI channels."""
    for ch in range(16):
        midiout.send_message([0xB0 + ch, 123, 0])  # 0xB0 = Control Change, 123 = All Notes Off

def main():
    if len(sys.argv) < 2:
        print("Usage: midi_to_iac.py <path_to_midi_file>")
        sys.exit(1)

    midi_file_path = sys.argv[1]

    # --- Load the MIDI file using Mido ---
    try:
        mid = mido.MidiFile(midi_file_path)
    except Exception as e:
        print(f"Failed to open MIDI file: {e}")
        sys.exit(1)

    # --- Create an rtmidi output object ---
    midiout = rtmidi.MidiOut()

    midiout.open_virtual_port("pedalboard")

    print(f"Playing MIDI file: {midi_file_path}")
    start_time = time.time()

    try:
        # --- Real-time Playback ---
        for msg in mid.play():
            if not msg.is_meta:
                midi_bytes = msg.bytes()
                midiout.send_message(midi_bytes)
                # Optional: log
                # print("Sent:", msg)

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
