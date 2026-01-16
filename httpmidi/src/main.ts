import express, { type Request, type Response } from "express";
import midi from "midi";
import cors from 'cors';
import fs from 'fs';
import TOML from 'smol-toml'
import alsaVolume from "alsa-volume";

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

app.get("/config", (req: Request, res: Response) => {
  const toml = fs.readFileSync('../Config.toml', 'utf8')
  const { synth } =  TOML.parse(toml)
  res.json(synth);
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