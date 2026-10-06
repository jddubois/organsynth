module.exports = {
  apps: [
    {
      // The synth, MIDI from the piano and pedalboard, and the web app (ports 8080 and 5173).
      name: "organsynth",
      script: "npm",
      args: "start",
      cwd: "/home/patch/organsynth/synth",
      interpreter: "none",
    },
  ],
};
