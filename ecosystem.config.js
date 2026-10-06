module.exports = {
  apps: [
    {
      // The synth, MIDI from the piano and pedalboard, and the web app (ports 8080 and 5173).
      name: "organsynth",
      script: "npm",
      args: "start",
      cwd: "/home/patch/organsynth/synth",
      interpreter: "none",
      // All four cores render audio (GrandOrgue no longer competes for them).
      // JACK_NO_START_SERVER: never let libjack start its own jackd when the system JACK is
      // down (e.g. at boot or while it restarts); fail instead and let pm2 retry.
      env: { THREADS: "4", JACK_NO_START_SERVER: "1" },
      restart_delay: 2000,
    },
  ],
};
