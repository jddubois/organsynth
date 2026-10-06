import { useCallback, useEffect, useState } from "react";
import "./App.css";

type Mode = "organ" | "piano";

interface State {
  mode: Mode;
  organ: string;
  organs: { id: string; name: string }[];
  registration: number;
  registrations: { name: string; manual: string[]; pedal: string[] }[];
  pianoPreset: string;
  pianoPresets: string[];
  devices: { piano: boolean; pedalboard: boolean };
  cpu: number;
  overloaded: boolean;
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

  const run = useCallback(async (path: string, body?: object) => {
    try {
      setState(await api(path, body));
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
          <select
            value={state.organ}
            onChange={(e) => {
              setState({ ...state, organ: e.target.value });
              run("organ", { organ: e.target.value });
            }}
            className="mb-5 w-full rounded-xl bg-organ-surface border-2 border-organ-border px-4 py-3
                       text-lg text-organ-text font-medium appearance-none cursor-pointer"
          >
            {state.organs.map((o) => (
              <option key={o.id} value={o.id}>
                {o.name}
              </option>
            ))}
          </select>

          <h2 className={heading}>PRESETS</h2>
          <div className="grid grid-cols-2 gap-4 flex-1 min-h-[18rem]">
            {state.registrations.map((r, i) => (
              <button
                key={i}
                onClick={() => {
                  setState({ ...state, registration: i });
                  run("registration", { registration: i });
                }}
                className={`${tile(state.registration === i)} flex flex-col items-center justify-center gap-1 px-2`}
              >
                <span className="text-3xl font-bold">{i + 1}</span>
                <span className="text-xs font-medium opacity-70 leading-tight text-center line-clamp-2">
                  {r.manual.join(" · ")}
                </span>
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

      {/* Status */}
      <div className="flex justify-center gap-4 text-xs text-organ-text-muted">
        <span>Piano {state.devices.piano ? "●" : "○"}</span>
        <span>Pedalboard {state.devices.pedalboard ? "●" : "○"}</span>
        <span className={state.overloaded ? "text-red-400" : ""}>CPU {Math.round(state.cpu * 100)}%</span>
        {error && <span className="text-red-400">{error}</span>}
      </div>
    </div>
  );
}

export default App;
