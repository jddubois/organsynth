import express, { type Request, type Response } from "express";
import midi from "midi";
import cors from 'cors';
import alsaVolume from "alsa-volume";

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

const PRESET_DEFAULTS = [
  { midi_channel: 1, channel_name: 'Manual',     preset_name: 'principal'          },
  { midi_channel: 2, channel_name: 'Pedalboard', preset_name: 'pedalboard_default' },
];

declare global {
  namespace Express {
    interface Request {
      midi: midi.Output;
    }
  }
}

const app = express();
const PORT = 8080;
const HOST = '0.0.0.0'

const output = new midi.Output();
output.openVirtualPort("httpmidi");

const midiMiddleware = async (req, res, next) => {
    req.midi = output;
    next();
};

app.use(cors());
app.use(express.json());
app.use(midiMiddleware);

app.post("/midi", (req: Request, res: Response) => {
  const message = req.body;
  console.log(JSON.stringify({ message }))
  req.midi.sendMessage(message);
  res.sendStatus(200)
});

app.get("/config", (_req: Request, res: Response) => {
  const presets: Record<string, unknown> = {};
  for (const [name, { cc, channels, displayName }] of Object.entries(PRESETS)) {
    presets[name] = { midi_identifier: cc, channels, display_name: displayName };
  }
  res.json({ presets, preset_defaults: PRESET_DEFAULTS });
});

app.listen(PORT, HOST, () => {
  console.log(`MIDI bridge server is running on http://${HOST}:${PORT}`);
});

app.get("/volume", (req: Request, res: Response) => {
  const card = "hw:2"; // adjust as needed (e.g., "default")
  const control = "Digital";
  const volume = alsaVolume.getVolume(card, control);
  const range = alsaVolume.getVolumeRange(card, control);
  console.log({ volume, range });
  const min = range.min ?? range[0] ?? 0;
  const max = range.max ?? range[1] ?? 100;
  const percent = Math.round(((volume - min) / (max - min)) * 100);
  res.json({ volume: Math.max(0, Math.min(100, percent)) });
});

app.post("/volume", (req: Request, res: Response) => {
  const card = "hw:2"; // adjust as needed (e.g., "default")
  const control = "Digital";
  const direction = (req.body?.direction || "").toString();
  const current = alsaVolume.getVolume(card, control);
  const range = alsaVolume.getVolumeRange(card, control);
  const min = range.min ?? range[0] ?? 0;
  const max = range.max ?? range[1] ?? 100;
  const step = Math.max(1, Math.round((max - min) * 0.05));
  const delta = direction === "up" ? step : direction === "down" ? -step : 0;
  const nextRaw = Math.max(min, Math.min(max, current + delta));
  alsaVolume.setVolume(card, control, nextRaw);
  const percent = Math.round(((nextRaw - min) / (max - min)) * 100);
  console.log({ direction, current, nextRaw, range, percent });
  res.json({ volume: Math.max(0, Math.min(100, percent)) });
});