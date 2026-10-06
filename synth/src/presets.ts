import type { DivisionName, OrganDefinition, OrganPreset } from '@supersynth/core';

// Six registrations per organ, soft to loud: soft flute, flutes, principal, principal chorus (or
// foundations), plenum, full organ. They are the organ's own presets (from supersynth), chosen
// so the same button means roughly the same thing on every organ. Solo registrations (a melody
// on one manual against another) are left out: the Donner is a single keyboard.

type Choice = string | { name: string; label: string; preset: OrganPreset };

const CHOICES: Record<string, Choice[]> = {
  burea: ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'full'],
  vcsl: [
    'chamber-8',
    'flutes',
    'chamber',
    { name: 'flutes-chamber', label: 'Flutes + Chamber', preset: { great: ['Flutes'], positive: ["Gedackt 8'", "Principal 4'"], pedal: ["Soft Bass 16'"] } },
    'full',
    { name: 'full-chamber', label: 'Full + Chamber', preset: { great: ['Full Organ'], positive: ["Gedackt 8'", "Principal 4'", 'Chorus'], pedal: ["Pedal 16' + 8'"] } },
  ],
  azzio: ['flute-8', 'flutes', 'principale', 'cornetto', 'ripieno', 'full'],
  cracow: ['flutes', 'flute-harmonique', 'principal', 'fonds', 'plein-jeu', 'grand-choeur'],
  'dluga-koscielna': ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'full'],
  friesach: ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'grand-choeur'],
  giubiasco: ['flute-8', 'flutes', 'principale', 'ripieno', 'pleno', 'full'],
  'green-positiv': ['flute-8', 'continuo', 'flute-2', 'principal', 'plenum', 'full'],
  harmonium: ['soft', 'celeste', 'diapason', '8-4', 'full', 'forte'],
  ledziny: ['flute-8', 'flutes', 'principal', 'foundations', 'principal-chorus', 'full'],
  lipiny: ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'full'],
  melcer: ['flute-8', 'flutes', 'sifflote', 'principal-chorus', 'plenum', 'full'],
  raszczyce: ['flute-8', 'flutes', 'prestant', 'principal-chorus', 'plenum', 'full'],
  'saint-jean-de-luz': ['jeux-doux', 'fonds', 'fonds-8-4', 'cornet', 'plein-jeu', 'grand-choeur'],
  skrzatusz: ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'full'],
  strassburg: ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'full'],
  szczecinek: ['flute-8', 'flutes', 'principal', 'principal-chorus', 'plenum', 'full'],
};

const LABELS: Record<string, string> = {
  'flute-8': 'Soft Flute',
  'chamber-8': 'Soft Flute',
  'flute-2': "Flute 8' + 2'",
  'flute-harmonique': 'Flûte Harmonique',
  'fonds-8-4': "Fonds 8' 4'",
  'grand-choeur': 'Grand Chœur',
  full: 'Full Organ',
  sifflote: 'Sifflöte',
  '8-4': "8' + 4'",
  celeste: 'Céleste',
};

const MANUALS: DivisionName[] = ['great', 'swell', 'positive'];

export interface Registration {
  /** The organ's preset name. */
  name: string;
  label: string;
  preset: OrganPreset;
}

/** Footage from a stop name: "Octave 4'" → 4, "Nasat 1 1/3'" → 1.33, "Mixture V" → 0. */
export function footage(name: string): number {
  const m = name.match(/(\d+)(?:\s+(\d+)\/(\d+))?\s*'/);
  if (!m) return 0;
  return Number(m[1]) + (m[2] ? Number(m[2]) / Number(m[3]) : 0);
}

/** The six registrations of an organ, adapted to be played from one keyboard. */
export function registrations(organ: OrganDefinition): Registration[] {
  const choices = CHOICES[organ.id] ?? Object.keys(organ.presets).slice(0, 6);
  return choices.map((choice, slot) => {
    const { name, label, preset } =
      typeof choice === 'string'
        ? { name: choice, label: LABELS[choice] ?? capitalize(choice), preset: organ.presets[choice] }
        : choice;
    if (!preset) throw new Error(`${organ.id} has no preset ${name}`);
    return { name, label, preset: oneKeyboard(organ, preset, slot < 2) };
  });
}

/**
 * The Donner plays the great and the pedalboard the pedal, so every other manual a preset draws
 * on is coupled to the great. A pedal without an 8' stop gets one: the piano's small speakers
 * barely reproduce 16' fundamentals (32–65 Hz), so a 16' alone all but disappears.
 */
function oneKeyboard(organ: OrganDefinition, preset: OrganPreset, soft: boolean): OrganPreset {
  const couple = structuredClone(preset.couple ?? {});
  const toGreat = (couple.great ??= []);
  for (const manual of MANUALS.slice(1)) {
    const coupled = toGreat.some((c) => (typeof c === 'string' ? c === manual : c.division === manual && !c.octave));
    if (preset[manual]?.length && !coupled) toGreat.push(manual);
  }
  if (!toGreat.length) delete couple.great;

  const pedalStops = organ.stops.filter((s) => s.division === 'pedal');
  let pedal = [...(preset.pedal ?? [])];
  if (!pedalStops.length) {
    couple.pedal = [...new Set([...(couple.pedal ?? []), 'great' as const])];
  } else if (!pedal.some((name) => Math.abs(footage(name) - 8) < 0.01)) {
    const eights = pedalStops.filter((s) => Math.abs(footage(s.name) - 8) < 0.01);
    const order = soft ? ['flute', 'principal', 'string', 'reed'] : ['principal', 'flute', 'string', 'reed'];
    const eight = order.map((family) => eights.find((s) => s.family === family)).find(Boolean);
    if (eight) pedal = [...pedal, eight.name];
  }
  return { ...preset, pedal, couple };
}

/** The stops a registration draws for the Donner, for display. One keyboard plays every manual,
 *  so the manuals aren't told apart; a name drawn on two manuals is listed once, as "Flöte 4' ×2". */
export function manualStops(preset: OrganPreset): string[] {
  const counts = new Map<string, number>();
  for (const manual of MANUALS) for (const name of preset[manual] ?? []) counts.set(name, (counts.get(name) ?? 0) + 1);
  return [...counts].map(([name, n]) => (n > 1 ? `${name} ×${n}` : name));
}

/** 'principal-chorus' → 'Principal Chorus'. */
function capitalize(name: string): string {
  return name.split('-').map((word) => word[0].toUpperCase() + word.slice(1)).join(' ');
}
