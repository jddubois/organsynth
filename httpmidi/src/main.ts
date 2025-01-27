import express, { type Request, type Response } from "express";
import midi from "midi";
import cors from 'cors';

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

app.listen(PORT, HOST, () => {
  console.log(`MIDI bridge server is running on http://${HOST}:${PORT}`);
});

