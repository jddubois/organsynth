// Measures every organ's registrations offline and writes src/balance.json:
//   npm run balance            (all organs)
//   npm run balance -- burea   (some)
// For each registration, the pedal's expression is set so a pedal note sits PEDAL_BELOW_DB under
// a manual triad (pedal not coupled to the manual), and each organ gets a trim so its principal chorus is as loud as the others'.
// Levels are measured above 100 Hz: what the piano's speakers actually reproduce.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { Synth, ORGANS, type OrganDefinition } from '@supersynth/core';
import { organDefinition, type Balance } from '../src/organ.ts';
import { registrations, type Registration } from '../src/presets.ts';

// 6 dB: by ear on the Donner's speakers (2 dB measured right but sounded 4 dB too loud).
const PEDAL_BELOW_DB = 6;
const MAX_TRIM_DB = 6;
const TRIM_SLOT = 3; // principal chorus
const FILE = new URL('../src/balance.json', import.meta.url);

function level(def: OrganDefinition, reg: Registration, part: 'great' | 'pedal', chords: number[][], expression = 1) {
  let power = 0;
  for (const notes of chords) {
    const synth = new Synth({ reverb: def.reverb ?? 'church', sampleRate: 48000 });
    const organ = synth.add(def, { preset: reg.preset, preload: false });
    organ.pedal.expression(expression);
    organ[part].play(notes, { duration: 2.5 });
    const x = synth.render(2.5).left.subarray(24000);
    synth.close();
    // second-order 100 Hz high-pass (two one-pole stages)
    const k = Math.exp((-2 * Math.PI * 100) / 48000);
    let x1 = 0, y1 = 0, x2 = 0, y2 = 0, sum = 0;
    for (const v of x) {
      const y = k * (y1 + v - x1); x1 = v; y1 = y;
      const z = k * (y2 + y - x2); x2 = y; y2 = z;
      sum += z * z;
    }
    power += sum / x.length;
  }
  return 10 * Math.log10(power / chords.length);
}

const db = (gain: number) => 20 * Math.log10(gain);
const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

const result: Record<string, Balance> = existsSync(FILE) ? JSON.parse(readFileSync(FILE, 'utf8')) : {};
const ids = process.argv.slice(2).length ? process.argv.slice(2) : Object.keys(ORGANS);
const chorus: Record<string, number> = {};

for (const id of ids) {
  const def = organDefinition(id);
  const pedal: number[] = [];
  for (const [slot, reg] of registrations(def).entries()) {
    const manual = level(def, reg, 'great', [[60, 64, 67], [55, 59, 62]]);
    if (slot === TRIM_SLOT) chorus[id] = manual;
    const target = manual - PEDAL_BELOW_DB;
    // Twice: when the pedal is coupled to the manual, the expression only scales part of it.
    let e = 1;
    for (let i = 0; i < 2; i++) e = clamp(e * 10 ** ((target - level(def, reg, 'pedal', [[36], [43]], e)) / 20), 0.05, 1);
    const final = level(def, reg, 'pedal', [[36], [43]], e);
    pedal.push(Number(e.toFixed(3)));
    console.log(`${id} ${reg.label}: manual ${manual.toFixed(1)} dB, pedal ${final.toFixed(1)} dB at ${db(e).toFixed(1)} dB${e === 1 && final < target - 1 ? '  (pedal too weak even at full)' : ''}`);
  }
  result[id] = { trim: result[id]?.trim ?? 0, pedal };
}

// Trims are relative to the median organ, so they're only recomputed when every organ is measured.
if (ids.length === Object.keys(ORGANS).length) {
  const levels = Object.values(chorus).sort((a, b) => a - b);
  const median = levels[levels.length >> 1];
  for (const id of ids) result[id].trim = Number(clamp(median - chorus[id], -MAX_TRIM_DB, MAX_TRIM_DB).toFixed(1));
} else {
  console.log('Trims kept: run without arguments to re-measure every organ and update them.');
}
writeFileSync(FILE, JSON.stringify(result, null, 2) + '\n');
console.log(`Wrote ${FILE.pathname}`);
