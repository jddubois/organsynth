import midi from '@julusian/midi';

export type Role = 'piano' | 'pedalboard';

const PATTERNS: Record<Role, RegExp> = {
  piano: /piano/i,
  pedalboard: /teensy/i,
};

const POLL_MS = 1000;
const LOCAL_CONTROL = 122;

/**
 * Keeps the piano and pedalboard MIDI inputs open across unplugging and power cycles, and
 * tells the piano "Local Control Off" every second, so its keys only send MIDI and its own
 * sound engine stays silent (the piano forgets the setting when it is switched off).
 */
export class Devices {
  private inputs = new Map<Role, { input: midi.Input; name: string }>();
  private pianoOut: { output: midi.Output; name: string } | null = null;
  private probeIn = new midi.Input();
  private probeOut = new midi.Output();
  private timer: NodeJS.Timeout | null = null;

  constructor(
    private onMessage: (role: Role, message: number[]) => void,
    private onDisconnect: (role: Role) => void,
  ) {}

  start() {
    this.poll();
    this.timer = setInterval(() => this.poll(), POLL_MS);
  }

  connected(): Record<Role, boolean> {
    return { piano: this.inputs.has('piano'), pedalboard: this.inputs.has('pedalboard') };
  }

  close() {
    if (this.timer) clearInterval(this.timer);
    for (const role of [...this.inputs.keys()]) this.closeInput(role);
    this.pianoOut?.output.closePort();
    this.pianoOut = null;
  }

  private poll() {
    const inputs = portNames(this.probeIn);
    for (const role of Object.keys(PATTERNS) as Role[]) {
      const index = inputs.findIndex((n) => PATTERNS[role].test(n));
      const open = this.inputs.get(role);
      if (open && open.name !== inputs[index]) this.closeInput(role);
      if (index >= 0 && !this.inputs.has(role)) this.openInput(role, index, inputs[index]);
    }

    const outputs = portNames(this.probeOut);
    const index = outputs.findIndex((n) => PATTERNS.piano.test(n));
    if (this.pianoOut && this.pianoOut.name !== outputs[index]) {
      this.pianoOut.output.closePort();
      this.pianoOut = null;
    }
    if (index >= 0 && !this.pianoOut) {
      const output = new midi.Output();
      output.openPort(index);
      this.pianoOut = { output, name: outputs[index] };
      console.log(`Sending Local Control Off to ${outputs[index]}`);
    }
    if (this.pianoOut) {
      for (let channel = 0; channel < 16; channel++) {
        this.pianoOut.output.sendMessage([0xb0 | channel, LOCAL_CONTROL, 0]);
      }
    }
  }

  private openInput(role: Role, index: number, name: string) {
    const input = new midi.Input();
    input.on('message', (_delta, message) => this.onMessage(role, message));
    input.openPort(index);
    this.inputs.set(role, { input, name });
    console.log(`Connected ${role}: ${name}`);
  }

  private closeInput(role: Role) {
    const open = this.inputs.get(role);
    if (!open) return;
    open.input.closePort();
    this.inputs.delete(role);
    this.onDisconnect(role);
    console.log(`Disconnected ${role}: ${open.name}`);
  }
}

function portNames(port: midi.Input | midi.Output): string[] {
  const names: string[] = [];
  for (let i = 0; i < port.getPortCount(); i++) names.push(port.getPortName(i));
  return names;
}
