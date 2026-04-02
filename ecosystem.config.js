module.exports = {
    apps: [
      {
        name: "stopmanager",
        script: "npm",
        args: "start",
        cwd: "/home/patch/organsynth/stopmanager",
        interpreter: "none",
      },
      {
        name: "httpmidi",
        script: "npm",
        args: "start",
        cwd: "/home/patch/organsynth/httpmidi",
        interpreter: "none",
      },
      {
        name: "organsynth",
        script: "npm",
        args: "start",
        cwd: "/home/patch/organsynth/synth",
        interpreter: "none",
      },
      {
        name: "a2jmidid",
        script: "a2jmidid",
        args: "-e",
      },
    ],
  };
  