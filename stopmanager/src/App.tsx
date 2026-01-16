import { useEffect, useState } from "react";
import "./App.css";
import { useConfig } from "./config";
import _ from "lodash";

function sendMidi(message: Array<number>) {
  fetch("http://192.168.1.21:8080/midi", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(message),
  });
}

function createCCMessage(channel: number, controlNumber: number) {
  // Validate inputs
  if (channel < 1 || channel > 16) {
    throw new Error("Channel must be between 1 and 16.");
  }
  if (controlNumber < 0 || controlNumber > 127) {
    throw new Error("Control number must be between 0 and 127.");
  }
  // MIDI channels are 0-based in the protocol (0-15)
  const statusByte = 0xb0 | (channel - 1);

  // Return the MIDI message as a 3-byte Uint8Array
  return [statusByte, controlNumber, 127];
}

function sendCC(channel: number, controlNumber: number) {
  const message = createCCMessage(channel, controlNumber);
  sendMidi(message);
}

function App() {

  const [volume, setVolume] = useState<number>(50);
  const [volumeLoading, setVolumeLoading] = useState<boolean>(true);

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
  if (loading) {
    return <p>Loading...</p>;
  }
  if (error) {
    return <p>Error: {error}</p>;
  }

  console.log(config);

  return (
    <>
      <div className="card">
        <label>Volume: {volume}%</label>
        <div style={{ display: "flex", gap: 12, justifyContent: "center", alignItems: "center" }}>
          <button onClick={() => adjustVolume("down")} disabled={volumeLoading || volume <= 0}>-</button>
          <button onClick={() => adjustVolume("up")} disabled={volumeLoading || volume >= 100}>+</button>
        </div>
      </div>
      {_.map(config?.preset_defaults, (preset_default) => {
        return (
          <div className="card">
            <p>{preset_default.channel_name}</p>
            {_(config.presets)
              .filter((preset) => {
                return preset.channels.includes(preset_default.midi_channel);
              })
              .map((preset) => {
                return (
                  <button
                    onClick={() =>
                      sendCC(
                        preset_default.midi_channel,
                        preset.midi_identifier,
                      )
                    }
                  >
                    {preset.display_name}
                  </button>
                );
              })
              .value()}
          </div>
        );
      })}
    </>
  );
}

export default App;
