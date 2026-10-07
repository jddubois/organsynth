// Turn the piano's own sound off or on from any computer it's plugged into:
//   npm run local-control -- off   (keys only send MIDI; what the synth service does every second)
//   npm run local-control -- on    (back to normal)
import { Synth } from '@supersynth/core';

const on = (process.argv[2] ?? 'off').toLowerCase() === 'on';
const synth = new Synth();
const name = synth.listMidiOutputs().find((n) => /piano|donner/i.test(n));
if (!name) {
  console.error('Piano MIDI port not found');
  process.exit(1);
}
const bytes = Array.from({ length: 16 }, (_, ch) => [0xb0 | ch, 122, on ? 127 : 0, 0xb0 | ch, 123, 0]).flat(); // Local Control, All Notes Off
synth.sendMidi(name, bytes);
synth.close();
console.log(`Local Control ${on ? 'ON' : 'OFF'} sent to ${name}`);
