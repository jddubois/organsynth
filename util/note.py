import gpiod
import time
import rtmidi

CONSUMER = "note"
CHIP = "gpiochip4"
SENSOR_PIN = 17

chip = gpiod.Chip(CHIP)
line = chip.get_line(SENSOR_PIN)
line.request(CONSUMER, gpiod.Line.DIRECTION_INPUT)
line.set_flags(gpiod.Line.BIAS_PULL_DOWN)



midiout = rtmidi.MidiOut()
# available_ports = midiout.get_ports()
midiout.open_virtual_port("pedalboard")
# if available_ports:
#     midiout.open_port(0)
# else:
#     midiout.open_virtual_port("My virtual output")

def handle_value_change(value):
    if value == 1:
         event = (0x90, 60, 100)
         midiout.send_message(event)
         print("Note on")
    else:
        event = (0x80, 60, 100)
        midiout.send_message(event)
        print("Note off")

value = line.get_value()
while True:
        next_value = line.get_value()
        if next_value != value:
            value = next_value
            handle_value_change(value)
        time.sleep(0.001)