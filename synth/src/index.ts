import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Synth, ORGANS, type Instrument, type Organ, type OrganDefinition } from '@supersynth/core';
import { Devices, type Role } from './devices.ts';
import { REGISTRATIONS, resolveRegistration } from './presets.ts';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const UI_DIR = path.resolve(HERE, '../../stopmanager/dist');
// 5173 is where the phone's bookmark points (the old Vite dev server).
const PORTS = (process.env.PORTS ?? '8080,5173').split(',').map(Number);
const STATE_FILE = process.env.STATE_FILE ?? path.join(homedir(), '.organsynth.json');
// The Pi's JACK server owns the DAC; elsewhere use the default device.
const BACKEND = (process.env.AUDIO_BACKEND ?? (process.platform === 'linux' ? 'jack' : 'auto')) as 'jack' | 'auto';
// Render threads; 'auto' is one per core but one (3 on a Pi 5).
const THREADS = process.env.THREADS ? Number(process.env.THREADS) : 'auto';

type Mode = 'organ' | 'piano';
interface State {
  mode: Mode;
  organ: string;
  /** Registration index, 0–5. */
  registration: number;
  pianoPreset: string;
}

const DEFAULTS: State = { mode: 'organ', organ: 'friesach', registration: 0, pianoPreset: 'default' };
const state: State = { ...DEFAULTS, ...readState() };

function readState(): Partial<State> {
  try {
    return JSON.parse(readFileSync(STATE_FILE, 'utf8'));
  } catch {
    return {};
  }
}

function saveState() {
  try {
    writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
  } catch (e) {
    console.error('Could not save state:', (e as Error).message);
  }
}

// ── Synth ────────────────────────────────────────────────────────────────────

const synth = new Synth({ backend: BACKEND, threads: THREADS, overloadGuard: true, reverb: false });
synth.on('error', (e: Error) => console.error('synth error:', e.message));

// Only organs whose packages are installed can be picked.
const organs: { id: string; name: string }[] = [];
for (const [id, def] of Object.entries(ORGANS) as [string, OrganDefinition][]) {
  try {
    await synth.load(def);
    organs.push({ id, name: def.name });
  } catch {
    // package not installed
  }
}
if (!organs.some((o) => o.id === state.organ)) state.organ = organs[0].id;

const piano: Instrument = synth.add('grand-piano');
if (!(state.pianoPreset in piano.presets())) state.pianoPreset = 'default';
piano.preset(state.pianoPreset);

let organ: Organ = loadOrgan(state.organ);

function loadOrgan(id: string): Organ {
  const def = ORGANS[id as keyof typeof ORGANS] as OrganDefinition;
  const started = Date.now();
  const added = synth.add(def, { preset: resolveRegistration(def, state.registration) });
  console.log(`Organ ${id} ready to play in ${Date.now() - started} ms`);
  added.ready.then(
    () => console.log(`Organ ${id}: every stop loaded in ${Date.now() - started} ms`),
    (e: Error) => console.error(`Organ ${id}: ${e.message}`),
  );
  return added;
}

function applyRoom() {
  // Each instrument's own room: the organ recordings already carry their church.
  const room = state.mode === 'organ' ? (organ.definition.reverb ?? 'church') : (piano.definition.reverb ?? 'hall');
  synth.set({ reverb: room });
}

applyRoom();
await synth.start();
console.log(`Audio running (${BACKEND}) at ${synth.sampleRate} Hz on ${synth.threads} threads`);

// ── MIDI ─────────────────────────────────────────────────────────────────────

function handleMidi(role: Role, [status, data1, data2]: number[]) {
  const type = status & 0xf0;
  const target = state.mode === 'piano' ? (role === 'piano' ? piano : null) : role === 'piano' ? organ.great : organ.pedal;
  if (!target) return; // the pedalboard is silent in piano mode

  if (type === 0x90 && data2 > 0) target.noteOn(data1, data2);
  else if (type === 0x80 || type === 0x90) target.noteOff(data1);
  else if (type === 0xb0 && target === piano && data1 < 120) {
    if (data1 === 64) piano.sustain(data2 >= 64);
    else piano.controlChange(data1, data2);
  }
}

const devices = new Devices(handleMidi, (role) => {
  // Release whatever the unplugged device was holding.
  if (role === 'piano') {
    piano.allNotesOff();
    piano.sustain(false);
    organ.great.allNotesOff();
  } else organ.pedal.allNotesOff();
});
devices.start();

// ── Actions ──────────────────────────────────────────────────────────────────

function setMode(mode: Mode) {
  if (mode === state.mode) return;
  synth.allNotesOff();
  piano.sustain(false);
  state.mode = mode;
  applyRoom();
}

function setOrgan(id: string) {
  if (!organs.some((o) => o.id === id)) throw new HttpError(400, `Unknown organ ${id}`);
  if (id === organ.definition.id) return;
  synth.remove(organ);
  state.organ = id;
  organ = loadOrgan(id);
  applyRoom();
}

function setRegistration(index: number) {
  if (!(index in REGISTRATIONS)) throw new HttpError(400, `Unknown preset ${index + 1}`);
  state.registration = index;
  organ.preset(resolveRegistration(organ.definition, index));
}

function setPianoPreset(name: string) {
  if (!(name in piano.presets())) throw new HttpError(400, `Unknown piano preset ${name}`);
  state.pianoPreset = name;
  piano.preset(name);
}

async function snapshot() {
  const def = organ.definition;
  return {
    mode: state.mode,
    organ: state.organ,
    organs,
    registration: state.registration,
    registrations: REGISTRATIONS.map((r, i) => {
      const preset = resolveRegistration(def, i);
      return { name: r.name, manual: [...preset.great!, ...preset.swell!, ...preset.positive!], pedal: preset.pedal };
    }),
    pianoPreset: state.pianoPreset,
    pianoPresets: Object.keys(piano.presets()),
    devices: devices.connected(),
    cpu: synth.cpuLoad,
    voices: synth.activeVoices,
    overloaded: synth.guardActive,
    guard: synth.guardStats,
  };
}

// ── HTTP ─────────────────────────────────────────────────────────────────────

class HttpError extends Error {
  constructor(readonly status: number, message: string) {
    super(message);
  }
}

const ACTIONS: Record<string, (body: any) => void | Promise<void>> = {
  '/api/mode': (b) => setMode(b.mode === 'piano' ? 'piano' : 'organ'),
  '/api/organ': (b) => setOrgan(String(b.organ)),
  '/api/registration': (b) => setRegistration(Number(b.registration)),
  '/api/piano-preset': (b) => setPianoPreset(String(b.preset)),
};

const MIME: Record<string, string> = {
  '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.ico': 'image/x-icon', '.json': 'application/json',
};

async function handle(req: IncomingMessage, res: ServerResponse) {
  const url = new URL(req.url ?? '/', 'http://localhost');
  if (url.pathname.startsWith('/api/')) {
    if (req.method === 'POST') {
      const action = ACTIONS[url.pathname];
      if (!action) throw new HttpError(404, 'Not found');
      let body = '';
      for await (const chunk of req) body += chunk;
      await action(body ? JSON.parse(body) : {});
      saveState();
    } else if (url.pathname !== '/api/state') throw new HttpError(404, 'Not found');
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify(await snapshot()));
    return;
  }

  // The built web app; unknown paths get index.html.
  let file = path.join(UI_DIR, path.normalize(url.pathname).replace(/^(\.\.[/\\])+/, ''));
  if (!file.startsWith(UI_DIR) || !existsSync(file) || url.pathname === '/') file = path.join(UI_DIR, 'index.html');
  if (!existsSync(file)) throw new HttpError(503, 'Web app not built: run npm run build in stopmanager/');
  res.writeHead(200, { 'Content-Type': MIME[path.extname(file)] ?? 'application/octet-stream' });
  res.end(readFileSync(file));
}

function onRequest(req: IncomingMessage, res: ServerResponse) {
  handle(req, res).catch((e) => {
    const status = e instanceof HttpError ? e.status : 500;
    if (status === 500) console.error(e);
    res.writeHead(status, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: e.message }));
  });
}
const servers = PORTS.map((port) =>
  createServer(onRequest).listen(port, '0.0.0.0', () => console.log(`Web app on http://0.0.0.0:${port}`)),
);

function shutdown() {
  devices.close();
  servers.forEach((s) => s.close());
  synth.close();
  process.exit(0);
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
