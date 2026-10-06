import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";
import { FitText } from "./FitText";

type Mode = "organ" | "piano";

interface State {
  mode: Mode;
  organ: string;
  organs: { id: string; name: string }[];
  registration: number;
  registrations: { label: string; manual: string[]; pedal: string[] }[];
  room: string;
  rooms: string[];
  pedalCoupled: boolean;
  hasPedal: boolean;
  pianoPreset: string;
  pianoPresets: string[];
}

// Served by the synth itself, so the API is on the same origin.
async function api(path: string, body?: object): Promise<State> {
  const response = await fetch(`/api/${path}`, {
    method: body ? "POST" : "GET",
    headers: body ? { "Content-Type": "application/json" } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  });
  const data = await response.json();
  if (!response.ok) throw new Error(data.error ?? response.statusText);
  return data;
}

// 'concert-hall' → 'Concert Hall'
const roomLabel = (room: string) =>
  room === "auto"
    ? "Default"
    : room === "off"
      ? "None"
      : room.split("-").map((word) => word[0].toUpperCase() + word.slice(1)).join(" ");

/** A chevron centered in its box ("left", "right" or "down"). */
function Chevron({ direction, className = "" }: { direction: "left" | "right" | "down"; className?: string }) {
  const points = { left: "15 5 8 12 15 19", right: "9 5 16 12 9 19", down: "6 9 12 15 18 9" }[direction];
  return (
    <svg viewBox="0 0 24 24" aria-hidden className={`block ${className}`} fill="none" stroke="currentColor"
         strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round">
      <polyline points={points} />
    </svg>
  );
}

const tile = (active: boolean) => `
  rounded-2xl transition-all duration-200 cursor-pointer select-none active:scale-95
  ${
    active
      ? "bg-amber-glow text-organ-bg shadow-[0_0_20px_rgba(212,162,78,0.5)] border-2 border-amber-bright/50"
      : "bg-organ-surface-light text-organ-text-muted border-2 border-organ-border hover:border-organ-text-muted/40 hover:text-organ-text/80 active:bg-organ-surface"
  }`;

function App() {
  const [state, setState] = useState<State | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Bumped by every action, so a poll that started before a tap can't undo it.
  const actions = useRef(0);

  const run = useCallback(async (path: string, body?: object) => {
    const seq = body ? ++actions.current : actions.current;
    try {
      const next = await api(path, body);
      if (seq !== actions.current) return;
      setState(next);
      setError(null);
    } catch (err) {
      setError((err as Error).message);
    }
  }, []);

  useEffect(() => {
    run("state");
    const timer = setInterval(() => run("state"), 3000);
    return () => clearInterval(timer);
  }, [run]);

  if (!state) {
    return (
      <div className="fixed inset-0 bg-organ-bg flex items-center justify-center">
        <p className="text-organ-text-muted text-lg">{error ?? "Loading..."}</p>
      </div>
    );
  }

  const heading = "text-sm font-semibold uppercase tracking-widest text-organ-text-muted mb-3";

  // An organ id, or a step through the list (wrapping around).
  const selectOrgan = (target: string | number) => {
    const ids = state.organs.map((o) => o.id);
    const organ =
      typeof target === "string"
        ? target
        : ids[(ids.indexOf(state.organ) + target + ids.length) % ids.length];
    setState({ ...state, organ });
    run("organ", { organ });
  };

  return (
    <div className="fixed inset-0 bg-organ-bg flex flex-col gap-5 p-4 overflow-y-auto">
      {/* Organ / piano */}
      <div className="grid grid-cols-2 gap-3 w-full max-w-lg mx-auto">
        {(["organ", "piano"] as Mode[]).map((mode) => (
          <button
            key={mode}
            onClick={() => {
              setState({ ...state, mode });
              run("mode", { mode });
            }}
            className={`${tile(state.mode === mode)} py-4 text-xl font-bold tracking-wide uppercase`}
          >
            {mode}
          </button>
        ))}
      </div>

      {state.mode === "organ" ? (
        <div className="flex-1 flex flex-col min-h-0 w-full max-w-lg mx-auto">
          <h2 className={heading}>ORGAN</h2>
          <div className="mb-5 flex gap-2">
            {[-1, 1].map((step) => (
              <button
                key={step}
                aria-label={step < 0 ? "Previous organ" : "Next organ"}
                onClick={() => selectOrgan(step)}
                className={`${tile(false)} w-14 shrink-0 flex items-center justify-center ${step > 0 ? "order-last" : ""}`}
              >
                <Chevron direction={step < 0 ? "left" : "right"} className="w-7 h-7" />
              </button>
            ))}
            {/* The full name, wrapping if needed, with the native picker invisibly on top. */}
            <div className="relative flex-1 min-w-0 rounded-2xl bg-organ-surface border-2 border-organ-border
                            flex items-center pl-4 pr-9 py-2 min-h-14">
              <span className="text-lg font-medium leading-tight text-organ-text">
                {state.organs.find((o) => o.id === state.organ)?.name}
              </span>
              <Chevron direction="down" className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 w-5 h-5 text-organ-text-muted" />
              <select
                value={state.organ}
                onChange={(e) => selectOrgan(e.target.value)}
                aria-label="Organ"
                className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
              >
                {state.organs.map((o) => (
                  <option key={o.id} value={o.id}>
                    {o.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <h2 className={heading}>PRESETS</h2>
          <div className="grid grid-cols-2 grid-rows-3 gap-4 flex-1 min-h-[18rem]">
            {state.registrations.map((r, i) => (
              <button
                key={i}
                onClick={() => {
                  setState({ ...state, registration: i });
                  run("registration", { registration: i });
                }}
                className={`${tile(state.registration === i)} min-h-0 overflow-hidden px-3 py-3 font-bold`}
              >
                <FitText text={r.label} />
              </button>
            ))}
          </div>
        </div>
      ) : (
        <div className="flex-1 flex flex-col min-h-0 w-full max-w-lg mx-auto">
          <h2 className={heading}>GRAND PIANO</h2>
          <div className="grid grid-cols-2 gap-4 flex-1 min-h-[18rem]">
            {state.pianoPresets.map((preset) => (
              <button
                key={preset}
                onClick={() => {
                  setState({ ...state, pianoPreset: preset });
                  run("piano-preset", { preset });
                }}
                className={`${tile(state.pianoPreset === preset)} text-lg font-bold capitalize`}
              >
                {preset.replace("-", " ")}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Manual → Pedal coupler (always on for an organ without pedal stops) */}
      {state.mode === "organ" && (
        <div className="w-full max-w-lg mx-auto flex items-center gap-3">
          <h2 className="text-sm font-semibold uppercase tracking-widest text-organ-text-muted w-16">PEDAL</h2>
          <button
            disabled={!state.hasPedal}
            onClick={() => {
              const on = !state.pedalCoupled;
              setState({ ...state, pedalCoupled: on });
              run("pedal-coupler", { on });
            }}
            className={`${tile(state.pedalCoupled)} flex-1 h-11 text-base font-bold disabled:opacity-60`}
          >
            {state.hasPedal ? "Manual → Pedal" : "Plays the manual"}
          </button>
        </div>
      )}

      {/* Room */}
      <div className="w-full max-w-lg mx-auto flex items-center gap-3">
        <h2 className="text-sm font-semibold uppercase tracking-widest text-organ-text-muted w-16">ROOM</h2>
        <div className="relative flex-1 rounded-2xl bg-organ-surface border-2 border-organ-border pl-4 pr-9 py-2.5">
          <span className="text-base font-medium text-organ-text">{roomLabel(state.room)}</span>
          <Chevron direction="down" className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 w-5 h-5 text-organ-text-muted" />
          <select
            value={state.room}
            onChange={(e) => {
              setState({ ...state, room: e.target.value });
              run("room", { room: e.target.value });
            }}
            aria-label="Room"
            className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
          >
            {state.rooms.map((room) => (
              <option key={room} value={room}>
                {roomLabel(room)}
              </option>
            ))}
          </select>
        </div>
      </div>

      {error && <p className="text-center text-sm text-red-400">{error}</p>}
    </div>
  );
}

export default App;
