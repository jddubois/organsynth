// Turn the piano's own sound off or on from any computer it's plugged into:
//   npm run local-control -- off   (keys only send MIDI; what the synth service does every second)
//   npm run local-control -- on    (back to normal)
import midi from '@julusian/midi';

const mode = (process.argv[2] ?? 'off').toLowerCase();
const out = new midi.Output();
const port = [...Array(out.getPortCount()).keys()].find((i) => /piano|donner/i.test(out.getPortName(i)));
if (port === undefined) {
  console.error('Piano MIDI port not found');
  process.exit(1);
}
out.openPort(port);
for (let channel = 0; channel < 16; channel++) {
  out.sendMessage([0xb0 | channel, 122, mode === 'on' ? 127 : 0]); // Local Control
  out.sendMessage([0xb0 | channel, 123, 0]); // All Notes Off
}
out.closePort();
console.log(`Local Control ${mode === 'on' ? 'ON' : 'OFF'} sent to ${out.getPortName(port)}`);
