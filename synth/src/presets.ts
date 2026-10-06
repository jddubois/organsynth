import type { OrganDefinition, OrganPreset, StopDefinition, StopFamily } from '@supersynth/core';

// The six numbered registrations from the stopmanager UI. They were GrandOrgue General
// combinations 1–6 on its demo organ (bound to CC 21–26); each stop is described here by
// family and footage so the same registration can be found on any supersynth organ.
//
// The Donner plays the great and the pedalboard plays the pedal. A manual stop found on
// another manual (e.g. the swell) is coupled to the great so it sounds from the Donner.
// Combinations 1 and 6 also drew reeds on an uncoupled swell, which the Donner never
// played, so they are left out. Every pedal registration also gets an 8' principal: the piano's
// small speakers barely reproduce the 16' and 8' flutes' fundamentals (32–130 Hz), so without
// it the pedal line all but disappears.

type Manual = 'great' | 'swell' | 'positive';

interface Want {
  family: StopFamily[];
  feet: number;
  /** Names to prefer among stops of the right family and footage. */
  prefer?: RegExp;
}

interface Registration {
  name: string;
  great: Want[];
  swell: Want[];
  pedal: Want[];
  /** Couple the swell to the great (GrandOrgue's II/I). */
  swellToGreat?: boolean;
  /** Couple the great to the pedal (I/P). */
  greatToPedal?: boolean;
}

const OPEN_FLUTE = /harm|hohl|holz|offen|otwart|travers|konzert|camino|portunal|jubal|flaut|fl[uû]te 8/i;
const STOPPED_FLUTE = /bourdon|bordun|bordone|gedac|gedeck|gedekt|kryty|rohr|röhr|roer|holpijp|cor de nuit/i;
const MIXTURE = /mix|plein|ripieno|fourn|scharf|scherp/i;
const OBOE = /hautbois|oboe|schalm|krum|krom|cromorne|fagot|basson|regale|clarinet/i;
const STRING = /gamb|viol|salic|fugara|aeoline|cello/i;

const principal = (feet: number): Want => ({ family: ['principal'], feet });
const openFlute = (feet: number): Want => ({ family: ['flute'], feet, prefer: OPEN_FLUTE });
const stoppedFlute = (feet: number): Want => ({ family: ['flute'], feet, prefer: STOPPED_FLUTE });
const gamba: Want = { family: ['string'], feet: 8, prefer: STRING };
const mixture: Want = { family: ['mixture'], feet: 0, prefer: MIXTURE };
const oboe: Want = { family: ['reed'], feet: 8, prefer: OBOE };

const subbass: Want = { family: ['flute'], feet: 16, prefer: /sub|sous|soub|untersatz|bordun|bourdon/i };
const pedalGedackt: Want = { family: ['flute'], feet: 8, prefer: STOPPED_FLUTE };
const contrabass: Want = { family: ['principal', 'string'], feet: 16, prefer: /contra|kontra|principal|prestant|violon/i };
const cello: Want = { family: ['string', 'principal'], feet: 8, prefer: /cell|violon|viol/i };
const pedalPrincipal8: Want = { family: ['principal', 'string', 'flute'], feet: 8, prefer: /oct|princip|prestant|choral/i };
const pedalFlute4: Want = { family: ['flute', 'principal'], feet: 4, prefer: /fl|choral/i };

export const REGISTRATIONS: Registration[] = [
  {
    name: 'Montre',
    great: [principal(8)],
    swell: [],
    pedal: [subbass, pedalGedackt, pedalPrincipal8],
  },
  {
    name: 'Fonds 8 4',
    great: [principal(8), principal(4)],
    swell: [stoppedFlute(8), gamba],
    pedal: [subbass, pedalGedackt, contrabass, pedalPrincipal8],
    swellToGreat: true,
  },
  {
    name: 'Flutes',
    great: [openFlute(8), { family: ['flute'], feet: 4 }],
    swell: [stoppedFlute(8), { family: ['flute'], feet: 4 }],
    pedal: [subbass, pedalGedackt, pedalPrincipal8, pedalFlute4],
    swellToGreat: true,
  },
  {
    name: 'Fonds',
    great: [principal(8), principal(4), stoppedFlute(16), openFlute(8), { family: ['flute'], feet: 4 }],
    swell: [stoppedFlute(8), { family: ['flute'], feet: 4 }, gamba],
    pedal: [subbass, pedalGedackt, contrabass, pedalPrincipal8, cello, pedalFlute4],
    swellToGreat: true,
  },
  {
    name: 'Plein jeu',
    great: [principal(8), principal(4), mixture, stoppedFlute(16)],
    swell: [stoppedFlute(8), oboe, { family: ['flute', 'principal'], feet: 2 }],
    pedal: [subbass, pedalGedackt, contrabass, pedalPrincipal8, cello, pedalFlute4],
    swellToGreat: true,
    greatToPedal: true,
  },
  {
    name: 'Grand 16',
    great: [principal(8), principal(4), stoppedFlute(16)],
    swell: [],
    pedal: [subbass, pedalGedackt, contrabass, pedalPrincipal8],
  },
];

/** Footage from a stop name: "Octave 4'" → 4, "Nasat 1 1/3'" → 1.33, "Mixture V" → 0. */
export function footage(name: string): number {
  const m = name.match(/(\d+)(?:\s+(\d+)\/(\d+))?\s*'/);
  if (!m) return 0;
  return Number(m[1]) + (m[2] ? Number(m[2]) / Number(m[3]) : 0);
}

const MANUALS: Manual[] = ['great', 'swell', 'positive'];

function pick(stops: StopDefinition[], want: Want, used: Set<string>): StopDefinition | undefined {
  const candidates = stops.filter(
    (s) =>
      !used.has(s.id) &&
      want.family.includes(s.family) &&
      (want.feet === 0 || Math.abs(footage(s.name) - want.feet) < 0.01) &&
      // a "Mixture" want must not pick a cornet or sesquialtera
      !(want.family.includes('mixture') && /corn|sesqui/i.test(s.name)),
  );
  // Prefer by family order, then by name.
  for (const family of want.family) {
    const ofFamily = candidates.filter((s) => s.family === family);
    const preferred = want.prefer && ofFamily.find((s) => want.prefer!.test(s.name));
    if (preferred) return preferred;
    if (ofFamily.length) return ofFamily[0];
  }
}

/** Resolve a numbered registration to a preset for this organ. */
export function resolveRegistration(organ: OrganDefinition, index: number): OrganPreset {
  const reg = REGISTRATIONS[index];
  const byDivision = (d: string) => organ.stops.filter((s) => s.division === d);
  const used = new Set<string>();
  const drawn: Record<Manual | 'pedal', string[]> = { great: [], swell: [], positive: [], pedal: [] };

  const place = (want: Want, order: (Manual | 'pedal')[]) => {
    for (const division of order) {
      const stop = pick(byDivision(division), want, used);
      if (stop) {
        used.add(stop.id);
        drawn[division].push(stop.name);
        return;
      }
    }
  };

  for (const want of reg.great) place(want, ['great', 'positive', 'swell']);
  if (reg.swellToGreat) for (const want of reg.swell) place(want, ['swell', 'positive', 'great']);
  // Nothing at all on the manuals (a small organ): fall back to any 8' stop.
  for (const want of [principal(8), openFlute(8), { family: ['flute', 'principal', 'string', 'reed'], feet: 8 } as Want]) {
    if (!MANUALS.some((m) => drawn[m].length)) place(want, MANUALS);
  }
  for (const want of reg.pedal) place(want, ['pedal']);

  const couple: OrganPreset['couple'] = {};
  const toGreat = (['swell', 'positive'] as Manual[]).filter((m) => drawn[m].length);
  if (toGreat.length) couple.great = toGreat;
  // An organ without pedal stops (the Green Positiv) plays the great from the pedalboard.
  if (reg.greatToPedal || !drawn.pedal.length) couple.pedal = ['great'];

  return {
    description: reg.name,
    great: drawn.great,
    swell: drawn.swell,
    positive: drawn.positive,
    pedal: drawn.pedal,
    couple,
  };
}

/** The stops a preset draws for the Donner, for display. One keyboard plays every manual, so the
 *  manuals aren't told apart; a name drawn on two manuals is listed once, as "Flöte 4' ×2". */
export function manualStops(preset: OrganPreset): string[] {
  const counts = new Map<string, number>();
  for (const name of [...preset.great!, ...preset.swell!, ...preset.positive!]) counts.set(name, (counts.get(name) ?? 0) + 1);
  return [...counts].map(([name, n]) => (n > 1 ? `${name} ×${n}` : name));
}
