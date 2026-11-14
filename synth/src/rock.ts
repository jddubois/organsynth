import { INSTRUMENTS, type Instrument, type InstrumentPreset, type Synth } from '@supersynth/core';

// The rock instruments, at the end of the organ list: supersynth's tonewheel organ through its
// rotary speaker, and its analog-style polysynth. Unlike the church organs each is a single
// instrument, not stops on divisions: the keyboard and the pedalboard both play it and its six
// presets are tones. The rock organ's speaker speed is set separately, from the phone or with
// the piano's sustain pedal; on the synth the sustain pedal sustains.

export type SpeakerSpeed = 'fast' | 'slow' | 'stop';
export const SPEAKER_SPEEDS: SpeakerSpeed[] = ['slow', 'fast', 'stop'];

export interface RockTone {
  label: string;
  /** One of the instrument's own presets, or parameters on top of its default sound. */
  preset: string | InstrumentPreset;
}

export interface RockInstrument {
  /** The supersynth instrument, and the id in the organ list. */
  id: string;
  name: string;
  tones: RockTone[];
  /** Has a rotary speaker: the Rotor row, and the sustain pedal as its footswitch. */
  speaker: boolean;
  /** Loudness trim (dB), as balance.json has for the church organs: brings the tone at
   *  `trimTone` to the level of the median organ's principal chorus, measured above 100 Hz
   *  like scripts/balance.ts (`npm run balance -- rock` prints the measurements). */
  trim: number;
}

export const ROCK_ORGAN: RockInstrument = {
  id: 'rock-organ',
  name: 'Rock Organ',
  // `driveLevel` keeps the driven tones about as loud as the clean one (measured on a triad).
  tones: [
    { label: 'Mellow', preset: { parameters: { brightness: -2.5 } } },
    { label: 'Warm', preset: { parameters: { brightness: -1 } } },
    { label: 'Classic', preset: { parameters: {} } },
    { label: 'Bright', preset: { parameters: { brightness: 1.5 } } },
    // Chords go through the one amplifier, so the drive that suits a single note turns a chord
    // to fuzz: these are half of what they were (4 and 9).
    { label: 'Overdrive', preset: { parameters: { drive: 2, driveTone: 7000, driveLevel: 0.56 } } },
    { label: 'Heavy', preset: { parameters: { drive: 4, driveTone: 6000, driveLevel: 0.35 } } },
  ],
  speaker: true,
  trim: 5.5,
};

export const ROCK_SYNTH: RockInstrument = {
  id: 'rock-synth',
  name: 'Rock Synth',
  tones: [
    { label: 'Pad', preset: 'pad' },
    { label: 'Strings', preset: 'strings' },
    { label: 'Brass', preset: 'default' },
    { label: 'Arena', preset: 'arena' },
    { label: 'Lead', preset: 'lead' },
    { label: 'Power', preset: 'power' },
  ],
  speaker: false,
  trim: 5,
};

export const ROCK_INSTRUMENTS = [ROCK_ORGAN, ROCK_SYNTH];

/** Whether the installed supersynth has this instrument (0.5.0 and later). */
export async function rockAvailable(synth: Synth, rock: RockInstrument): Promise<boolean> {
  if (!(rock.id in INSTRUMENTS)) return false;
  try {
    await synth.load(rock.id as keyof typeof INSTRUMENTS);
    return true;
  } catch {
    return false;
  }
}

export function addRock(synth: Synth, rock: RockInstrument): Instrument {
  return synth.add(rock.id as keyof typeof INSTRUMENTS);
}

/** Set a tone and, on the rock organ, the speaker's speed (its rotors keep turning through a
 *  change of tone). */
export function applyRockTone(rock: RockInstrument, instrument: Instrument, index: number, speaker: SpeakerSpeed) {
  const { preset } = rock.tones[index];
  if (typeof preset === 'string') instrument.preset(preset);
  else instrument.preset(rock.speaker ? { ...preset, parameters: { ...preset.parameters, leslie: speaker } } : preset);
}
