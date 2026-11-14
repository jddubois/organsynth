import { INSTRUMENTS, type Instrument, type InstrumentParameters, type Synth } from '@supersynth/core';

// The rock organ: supersynth's tonewheel organ through its rotary speaker. Unlike the church
// organs it is a single instrument, not stops on divisions: the keyboard and the pedalboard
// both play it, its six presets are tones (soft to loud, like the other organs' buttons) and
// the speaker's speed is set separately, from the phone or with the piano's sustain pedal.

export const ROCK_ID = 'rock-organ';
export const ROCK_NAME = 'Rock Organ';

export type SpeakerSpeed = 'fast' | 'slow' | 'stop';
export const SPEAKER_SPEEDS: SpeakerSpeed[] = ['slow', 'fast', 'stop'];

export interface RockTone {
  label: string;
  parameters: InstrumentParameters;
}

// `driveLevel` keeps the driven tones about as loud as the clean one (measured on a triad).
export const ROCK_TONES: RockTone[] = [
  { label: 'Mellow', parameters: { brightness: -2.5 } },
  { label: 'Warm', parameters: { brightness: -1 } },
  { label: 'Classic', parameters: {} },
  { label: 'Bright', parameters: { brightness: 1.5 } },
  { label: 'Overdrive', parameters: { drive: 4, driveTone: 7000, driveLevel: 0.42 } },
  { label: 'Heavy', parameters: { drive: 9, driveTone: 6000, driveLevel: 0.36 } },
];

// Loudness trim (dB), as balance.json has for the church organs: brings a Classic triad to the
// level of the median organ's principal chorus, measured above 100 Hz like scripts/balance.ts
// (`npm run balance -- rock-organ` prints the measurement).
export const ROCK_TRIM_DB = 5.5;

/** Whether the installed supersynth has the rock organ (0.5.0 and later). */
export async function rockAvailable(synth: Synth): Promise<boolean> {
  if (!(ROCK_ID in INSTRUMENTS)) return false;
  try {
    await synth.load(ROCK_ID as keyof typeof INSTRUMENTS);
    return true;
  } catch {
    return false;
  }
}

export function addRock(synth: Synth): Instrument {
  return synth.add(ROCK_ID as keyof typeof INSTRUMENTS);
}

/** Set a tone and the speaker's speed. The rotors keep turning through a change of tone. */
export function applyRockTone(rock: Instrument, index: number, speaker: SpeakerSpeed) {
  rock.preset({ parameters: { ...ROCK_TONES[index].parameters, leslie: speaker } });
}
