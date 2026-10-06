import { readFileSync } from 'node:fs';
import { ORGANS, type Organ, type OrganDefinition } from '@supersynth/core';
import { registrations, type Registration } from './presets.ts';

// Pedal stops play this much louder than recorded, as headroom: each registration then turns
// the pedal down (its expression) to sit level with the manual, from balance.json.
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

// Without measurements: +4 dB over the recording.
const DEFAULT_PEDAL = 10 ** ((4 - PEDAL_HEADROOM_DB) / 20);

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

/** Draw a registration and set the pedal's level for it, `offsetDb` above or below the
 *  measured balance (the player's Pedal setting; the pedal can't go above its headroom). */
export function applyRegistration(organ: Organ, regs: Registration[], index: number, offsetDb = 0) {
  organ.preset(regs[index].preset);
  applyPedal(organ, index, offsetDb);
}

export function applyPedal(organ: Organ, index: number, offsetDb: number) {
  const measured = balance[organ.definition.id]?.pedal[index] ?? DEFAULT_PEDAL;
  organ.pedal.expression(Math.min(1, measured * 10 ** (offsetDb / 20)));
}

/** The organ's loudness trim, as a linear gain. */
export function organTrim(id: string): number {
  return 10 ** ((balance[id]?.trim ?? 0) / 20);
}
