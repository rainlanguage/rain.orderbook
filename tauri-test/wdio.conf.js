const os = require('os')
const path = require('path')
const { spawn, spawnSync } = require('child_process')

let tauriDriver

exports.config = {
  specs: ['./test/specs/**/*.js'],
  maxInstances: 1,
  hostname: 'localhost',
  port: 4444,
  capabilities: [
    {
      maxInstances: 1,
      'tauri:options': {
        application: '../tauri-app/src-tauri/target/release/rain-orderbook',
      },
      browserName: "wry",
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },

  beforeSession: () =>
    (tauriDriver = spawn(
      path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
      [],
      { stdio: [null, process.stdout, process.stderr] }
    )),

  afterSession: () => tauriDriver.kill(),
}