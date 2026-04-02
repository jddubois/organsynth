import { Organ } from 'supersynth';
import midi from 'midi';

const PRESETS = {
  principal:          { cc: 21, channels: [1], displayName: 'Default'   },
  grand_jeu:          { cc: 22, channels: [1], displayName: 'Pleno'     },
  cornet:             { cc: 30, channels: [1], displayName: 'Cornet'    },
  mixture:            { cc: 23, channels: [1], displayName: 'Mixture'   },
  flute:              { cc: 24, channels: [1], displayName: 'Flute'     },
  trumpet:            { cc: 27, channels: [1], displayName: 'Trumpet'   },
  plein_jeu:          { cc: 28, channels: [1], displayName: 'Plein Jeu' },
  pedalboard_default: { cc: 20, channels: [2], displayName: 'Default'   },
  pedalboard_reed:    { cc: 29, channels: [2], displayName: 'Reed'      },
  pedalboard_flute:   { cc: 25, channels: [2], displayName: 'Flute'     },
  pedalboard_trumpet: { cc: 26, channels: [2], displayName: 'Trumpet'   },
};

const DEFAULTS = [
  { midi_channel: 1, preset_name: 'principal'          },
  { midi_channel: 2, preset_name: 'pedalboard_default' },
];

// Build CC→preset lookup per channel
const ccMap: Record<number, Record<number, string>> = {};
for (const [name, { cc, channels }] of Object.entries(PRESETS)) {
  for (const ch of channels) {
    ccMap[ch] ??= {};
    ccMap[ch][cc] = name;
  }
}

// One Organ instance per MIDI channel
const organs: Record<number, Organ> = {
  1: new Organ({ backend: 'jack' }),
  2: new Organ({ backend: 'jack' }),
};

// Activate default presets
for (const { midi_channel, preset_name } of DEFAULTS) {
  organs[midi_channel].activatePreset(preset_name);
}

await organs[1].start();
await organs[2].start();
console.log('OrganSynth running');

// MIDI input — virtual ALSA port bridged to JACK via a2jmidid
const input = new midi.Input();
input.openVirtualPort('organsynth');

input.on('message', (_delta, [status, byte1, byte2]) => {
  const type = status & 0xF0;
  const channel = (status & 0x0F) + 1;
  const organ = organs[channel];
  if (!organ) return;

  if (type === 0x90 && byte2 > 0) {
    organ.noteOn(byte1, byte2);
  } else if (type === 0x80 || (type === 0x90 && byte2 === 0)) {
    organ.noteOff(byte1);
  } else if (type === 0xB0) {
    const preset = ccMap[channel]?.[byte1];
    if (preset) byte2 > 63 ? organ.activatePreset(preset) : organ.deactivatePreset(preset);
  }
});

process.on('SIGINT', () => {
  input.closePort();
  organs[1].stop();
  organs[2].stop();
  process.exit(0);
});
