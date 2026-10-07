import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { Synth, ORGANS, type Instrument, type Organ, type OrganDefinition, type ReverbPreset } from '@supersynth/core';
import { applyRegistration, hasPedal, organDefinition, organRegistrations, organTrim } from './organ.ts';
import { manualStops, type Registration } from './presets.ts';

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
  /** Per mode: 'auto' (the instrument's own room), 'off' or a reverb preset. */
  room: Record<Mode, string>;
  /** The Manual → Pedal coupler. */
  pedalCoupled: boolean;
}

const ROOMS = ['auto', 'off', 'room', 'studio', 'chamber', 'hall', 'concert-hall', 'church', 'cathedral', 'plate'];

// Master volume per mode (0–1). At the same setting a plenum chord peaks ~15 dB above a
// fortissimo piano chord and runs into the limiter, so the organ plays 14 dB lower: the two
// modes then match and the piano's volume knob has a usable range.
const VOLUME: Record<Mode, number> = { organ: 0.1, piano: 0.5 };

const DEFAULTS: State = {
  mode: 'organ',
  organ: 'friesach',
  registration: 0,
  pianoPreset: 'default',
  room: { organ: 'auto', piano: 'auto' },
  pedalCoupled: false,
};
const saved = readState();
const state: State = { ...DEFAULTS, ...saved, room: { ...DEFAULTS.room, ...saved.room } };

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

// releaseCulling ends release tails once they are 80 dB under the music (an inaudible change, per
// supersynth's measurements), so the overload guard, which cuts tails audibly, rarely has to act.
const synth = new Synth({
  backend: BACKEND,
  threads: THREADS,
  overloadGuard: true,
  releaseCulling: { belowMixDb: 80, hold: 'smooth' },
  reverb: false,
});
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

let registrations: Registration[] = [];
let organ: Organ = loadOrgan(state.organ);

function loadOrgan(id: string): Organ {
  registrations = organRegistrations(id);
  state.registration = Math.min(state.registration, registrations.length - 1);
  const started = Date.now();
  const added = synth.add(organDefinition(id), { preset: registrations[state.registration].preset });
  applyRegistration(added, registrations, state.registration, state.pedalCoupled);
  console.log(`Organ ${id} ready to play in ${Date.now() - started} ms`);
  added.ready.then(
    () => console.log(`Organ ${id}: every stop loaded in ${Date.now() - started} ms`),
    (e: Error) => console.error(`Organ ${id}: ${e.message}`),
  );
  return added;
}

function applyRoom() {
  // The room picked for this mode, or the instrument's own (the organ recordings already carry
  // their church), and the level: per mode, and per organ so that every organ is as loud.
  const own = state.mode === 'organ' ? (organ.definition.reverb ?? 'church') : (piano.definition.reverb ?? 'hall');
  const room = state.room[state.mode];
  const reverb = room === 'auto' ? own : room === 'off' ? false : (room as ReverbPreset);
  const volume = VOLUME.organ * (state.mode === 'organ' ? organTrim(organ.definition.id) : 1);
  synth.set({ reverb, volume: state.mode === 'organ' ? volume : VOLUME.piano });
}

applyRoom();
await synth.start();
console.log(`Audio running (${BACKEND}) at ${synth.sampleRate} Hz on ${synth.threads} threads, real-time: ${synth.realtime}`);

// If the audio output dies (e.g. JACK restarted), exit so pm2 starts a fresh synth. 'stopped'
// covers the output failing; the engine clock standing still covers anything else.
synth.on('stopped', (e?: Error) => {
  console.error(`Audio output stopped${e ? `: ${e.message}` : ''}; restarting`);
  process.exit(1);
});
let lastClock = -1;
setInterval(() => {
  const clock = synth.currentTime;
  if (clock !== lastClock && synth.isRunning) {
    lastClock = clock;
    return;
  }
  console.error('Audio clock stopped; restarting');
  process.exit(1);
}, 3000);

// ── MIDI ─────────────────────────────────────────────────────────────────────

// The engine plays MIDI itself: one input per device (reconnected after unplugging or a power
// cycle), each routed to an organ division or the piano. The Donner plays on channel 1 (it also
// sends its pedals on channels 2 and 3), the pedalboard on channel 2.
const PIANO = 'piano';
const PEDALBOARD = 'teensy';
await synth.enableMidi(PIANO, { optional: true });
await synth.enableMidi(PEDALBOARD, { optional: true });

function routeMidi() {
  if (state.mode === 'organ') {
    piano.midi(false);
    // Notes only: the Donner's sustain pedal mustn't hold organ pipes.
    organ.midi(
      { great: { device: PIANO, channel: 1, controllers: false }, pedal: { device: PEDALBOARD, channel: 2, controllers: false } },
      { presets: false },
    );
  } else {
    organ.midi(false); // the pedalboard is silent in piano mode
    piano.midi({ device: PIANO, channel: 1 }); // with its pedals: sustain (half-pedal), sostenuto, soft
  }
}
routeMidi();

// Local Control Off, every second: the piano's keys then only send MIDI and its own sound stays
// off (it forgets the setting when switched off).
const LOCAL_OFF = Array.from({ length: 16 }, (_, ch) => [0xb0 | ch, 122, 0]).flat();
setInterval(() => synth.sendMidi(PIANO, LOCAL_OFF), 1000);
synth.on('midiDevice', ({ device, name, connected }: { device?: string; name: string | null; connected: boolean }) => {
  console.log(`${connected ? 'Connected' : 'Disconnected'} ${device}: ${name}`);
  if (connected && device === PIANO) synth.sendMidi(PIANO, LOCAL_OFF);
});

// ── Actions ──────────────────────────────────────────────────────────────────

function setMode(mode: Mode) {
  if (mode === state.mode) return;
  state.mode = mode;
  routeMidi(); // held keys and pedals are let go on the instrument they were playing
  applyRoom();
}

function setOrgan(id: string) {
  if (!organs.some((o) => o.id === id)) throw new HttpError(400, `Unknown organ ${id}`);
  if (id === organ.definition.id) return;
  synth.remove(organ);
  state.organ = id;
  organ = loadOrgan(id);
  routeMidi();
  applyRoom();
}

function setRegistration(index: number) {
  if (!(index in registrations)) throw new HttpError(400, `Unknown preset ${index + 1}`);
  state.registration = index;
  applyRegistration(organ, registrations, index, state.pedalCoupled);
}

function setPedalCoupled(on: boolean) {
  state.pedalCoupled = on;
  applyRegistration(organ, registrations, state.registration, on);
}

function setRoom(room: string) {
  if (!ROOMS.includes(room)) throw new HttpError(400, `Unknown room ${room}`);
  state.room[state.mode] = room;
  applyRoom();
}

function setPianoPreset(name: string) {
  if (!(name in piano.presets())) throw new HttpError(400, `Unknown piano preset ${name}`);
  state.pianoPreset = name;
  piano.preset(name);
}

async function snapshot() {
  return {
    mode: state.mode,
    organ: state.organ,
    organs,
    registration: state.registration,
    registrations: registrations.map((r) => ({ label: r.label, manual: manualStops(r.preset), pedal: r.preset.pedal })),
    room: state.room[state.mode],
    pedalCoupled: state.pedalCoupled || !hasPedal(organ),
    hasPedal: hasPedal(organ),
    rooms: ROOMS,
    pianoPreset: state.pianoPreset,
    pianoPresets: Object.keys(piano.presets()),
    devices: Object.fromEntries(synth.midiInputs().map((i: { device?: string; connected: boolean }) => [i.device === PIANO ? 'piano' : 'pedalboard', i.connected])),
    xruns: synth.xruns,
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
  '/api/room': (b) => setRoom(String(b.room)),
  '/api/pedal-coupler': (b) => setPedalCoupled(Boolean(b.on)),
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
  servers.forEach((s) => s.close());
  synth.close();
  process.exit(0);
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
