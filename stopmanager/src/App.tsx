import { useState } from "react";
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
