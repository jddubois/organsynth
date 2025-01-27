import { useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'


function sendMidi(message: Array<number>) {
  fetch('http://192.168.1.21:8080/midi', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(message),
  })
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
  const statusByte = 0xB0 | (channel - 1);

  // Return the MIDI message as a 3-byte Uint8Array
  return [statusByte, controlNumber, 127];
}

function sendCC(channel: number, controlNumber: number) {
  const message = createCCMessage(channel, controlNumber);
  sendMidi(message);
}

function App() {
  const [count, setCount] = useState(0)

  return (
    <>
      <div>
        <a href="https://vite.dev" target="_blank">
          <img src={viteLogo} className="logo" alt="Vite logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <h1>Vite + React</h1>
      <div className="card">
        <button onClick={() => setCount((count) => count + 1)}>
          count is {count}
        </button>
        <p>
          Edit <code>src/App.tsx</code> and save to test HMR
        </p>
      </div>
      <div className="card">
        <p>MANUAL PRESETS</p>
        <button onClick={() => sendCC(1, 21)}>
          DEFAULT
        </button>
        <button onClick={() => sendCC(1, 22)}>
          PLENO
        </button>
        <button onClick={() => sendCC(1, 23)}>
          MIXTURE
        </button>
        <button onClick={() => sendCC(1, 24)}>
          FLUTE
        </button>
      </div>
      <div className="card">
        <p>PEDAL PRESETS</p>
        <button onClick={() => sendCC(2, 20)}>
          DEFAULT
        </button>
        <button onClick={() => sendCC(2, 25)}>
          FLUTE
        </button>
      </div>
      <p className="read-the-docs">
        Click on the Vite and React logos to learn more
      </p>
    </>
  )
}

export default App
