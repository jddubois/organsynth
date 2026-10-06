import { readFileSync } from 'node:fs';
import { ORGANS, type Organ, type OrganDefinition } from '@supersynth/core';
import { registrations, withPedalCoupled, type Registration } from './presets.ts';

// Pedal stops play this much louder than recorded, as headroom: each registration then turns
// the pedal down (its expression) to sit just under the manual, from balance.json.
export const PEDAL_HEADROOM_DB = 18;

/** Per organ, measured by scripts/balance.ts: the pedal expression for each registration, and
 *  a trim in dB that brings the organ to the same loudness as the others. */
export interface Balance {
  trim: number;
  pedal: number[];
}

const BALANCE_FILE = new URL('./balance.json', import.meta.url);
const balance: Record<string, Balance> = (() => {
  try {
    return JSON.parse(readFileSync(BALANCE_FILE, 'utf8'));
  } catch {
    return {};
  }
})();

// Without measurements: the recorded level.
const DEFAULT_PEDAL = 10 ** (-PEDAL_HEADROOM_DB / 20);

export function organDefinition(id: string): OrganDefinition {
  const catalog = ORGANS[id as keyof typeof ORGANS] as OrganDefinition;
  return {
    ...catalog,
    stops: catalog.stops.map((s) => (s.division === 'pedal' ? { ...s, gain: (s.gain ?? 0) + PEDAL_HEADROOM_DB } : s)),
  };
}

export function organRegistrations(id: string): Registration[] {
  return registrations(organDefinition(id));
}

/** Draw a registration and set the pedal's level for it. The pedal also plays the manual stops
 *  when `pedalCoupled`, and always on an organ without pedal stops (the Green Positiv). */
export function applyRegistration(organ: Organ, regs: Registration[], index: number, pedalCoupled: boolean) {
  const { preset } = regs[index];
  organ.preset(pedalCoupled || !hasPedal(organ) ? withPedalCoupled(preset) : preset);
  organ.pedal.expression(balance[organ.definition.id]?.pedal[index] ?? DEFAULT_PEDAL);
}

export function hasPedal(organ: Organ): boolean {
  return organ.definition.stops.some((s) => s.division === 'pedal');
}

/** The organ's loudness trim, as a linear gain. */
export function organTrim(id: string): number {
  return 10 ** ((balance[id]?.trim ?? 0) / 20);
}
