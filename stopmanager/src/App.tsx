import { useEffect, useState } from "react";
import "./App.css";
import { useConfig } from "./config";

function sendMidi(message: Array<number>) {
  fetch("http://192.168.1.21:8080/midi", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(message),
  });
}

function createCCMessage(channel: number, controlNumber: number, value: number) {
  if (channel < 1 || channel > 16) {
    throw new Error("Channel must be between 1 and 16.");
  }
  if (controlNumber < 0 || controlNumber > 127) {
    throw new Error("Control number must be between 0 and 127.");
  }
  const statusByte = 0xb0 | (channel - 1);
  return [statusByte, controlNumber, value];
}

function sendCC(channel: number, controlNumber: number, value: number) {
  const message = createCCMessage(channel, controlNumber, value);
  sendMidi(message);
}

function App() {
  const [volume, setVolume] = useState<number>(50);
  const [volumeLoading, setVolumeLoading] = useState<boolean>(true);
  const [activePresets, setActivePresets] = useState<Record<number, Set<number>>>({});

  useEffect(() => {
    const fetchVolume = async () => {
      try {
        const response = await fetch("http://192.168.1.21:8080/volume");
        if (!response.ok) {
          throw new Error(`Failed to fetch volume: ${response.statusText}`);
        }
        const data = await response.json();
        if (typeof data.volume === "number") {
          setVolume(data.volume);
        }
      } catch (err) {
        console.error(err);
      } finally {
        setVolumeLoading(false);
      }
    };
    fetchVolume();
  }, []);

  const adjustVolume = async (direction: "up" | "down") => {
    try {
      const response = await fetch("http://192.168.1.21:8080/volume", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ direction }),
      });
      if (response.ok) {
        const data = await response.json();
        if (typeof data.volume === "number") {
          setVolume(data.volume);
        }
      }
    } catch (err) {
      console.error(err);
    }
  };

  const { config, loading, error } = useConfig();

  useEffect(() => {
    if (!config) return;
    const initial: Record<number, Set<number>> = {};
    for (const preset_default of config.preset_defaults) {
      const preset = config.presets[preset_default.preset_name];
      if (preset) {
        if (!initial[preset_default.midi_channel]) {
          initial[preset_default.midi_channel] = new Set();
        }
        initial[preset_default.midi_channel].add(preset.midi_identifier);
      }
    }
    setActivePresets(initial);
  }, [config]);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-organ-text-muted text-lg">Loading...</p>
      </div>
    );
  }
  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <p className="text-red-400 text-lg">Error: {error}</p>
      </div>
    );
  }

  const togglePreset = (channel: number, ccId: number) => {
    const channelActive = activePresets[channel] ?? new Set();
    const isActive = channelActive.has(ccId);

    if (isActive) {
      sendCC(channel, ccId, 0);
      const next = new Set(channelActive);
      next.delete(ccId);
      setActivePresets({ ...activePresets, [channel]: next });
    } else {
      sendCC(channel, ccId, 127);
      const next = new Set(channelActive);
      next.add(ccId);
      setActivePresets({ ...activePresets, [channel]: next });
    }
  };

  return (
    <div className="min-h-screen px-4 py-6 max-w-2xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <h1 className="text-2xl font-semibold tracking-tight text-organ-text">
          OrganSynth
        </h1>

        {/* Volume control */}
        <div className="flex items-center gap-3">
          <span className="text-sm text-organ-text-muted">Volume</span>
          <button
            onClick={() => adjustVolume("down")}
            disabled={volumeLoading || volume <= 0}
            className="w-9 h-9 rounded-lg bg-organ-surface border border-organ-border
                       text-organ-text font-semibold text-lg
                       hover:bg-organ-surface-light hover:border-amber-glow/40
                       disabled:opacity-30 disabled:cursor-not-allowed
                       transition-all duration-150 cursor-pointer"
          >
            -
          </button>
          <span className="text-sm text-organ-text tabular-nums w-10 text-center font-medium">
            {volume}%
          </span>
          <button
            onClick={() => adjustVolume("up")}
            disabled={volumeLoading || volume >= 100}
            className="w-9 h-9 rounded-lg bg-organ-surface border border-organ-border
                       text-organ-text font-semibold text-lg
                       hover:bg-organ-surface-light hover:border-amber-glow/40
                       disabled:opacity-30 disabled:cursor-not-allowed
                       transition-all duration-150 cursor-pointer"
          >
            +
          </button>
        </div>
      </div>

      {/* Channel sections */}
      {config?.preset_defaults.map((preset_default: any) => {
        const channelPresets = Object.values(config.presets).filter(
          (preset: any) => preset.channels.includes(preset_default.midi_channel)
        );

        return (
          <div
            key={preset_default.midi_channel}
            className="mb-6 rounded-xl bg-organ-surface border border-organ-border p-5"
          >
            <h2 className="text-sm font-semibold uppercase tracking-widest text-organ-text-muted mb-4">
              {preset_default.channel_name}
            </h2>
            <div className="flex flex-wrap gap-2.5">
              {channelPresets.map((preset: any) => {
                const isActive = activePresets[preset_default.midi_channel]?.has(
                  preset.midi_identifier
                );
                return (
                  <button
                    key={preset.midi_identifier}
                    onClick={() =>
                      togglePreset(
                        preset_default.midi_channel,
                        preset.midi_identifier
                      )
                    }
                    className={`
                      px-5 py-3 rounded-lg text-base font-medium
                      transition-all duration-200 cursor-pointer
                      ${
                        isActive
                          ? "bg-amber-glow text-organ-bg shadow-[0_0_12px_rgba(212,162,78,0.35)] border border-amber-bright/50"
                          : "bg-organ-surface-light text-organ-text-muted border border-organ-border hover:border-organ-text-muted/40 hover:text-organ-text/80"
                      }
                    `}
                  >
                    {preset.display_name}
                  </button>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export default App;
