#!/usr/bin/env node
if (!process.argv.slice(2).some(a => a === '--stdio' || a === '--node-ipc' || a.startsWith('--socket'))) {
  process.argv.push('--stdio');
}
require('../dist/server.js');
