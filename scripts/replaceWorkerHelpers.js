const fs = require('fs');
const path = require('path');

const entryPath = path.resolve(__dirname, '../wasm_dist/wasm.js');
let file = fs.readFileSync(entryPath, {
  encoding: 'utf-8',
});
let content = file.split('\n');
content[0] = `import { startWorkers } from '../scripts/workerHelpers.js';`;
fs.writeFileSync(entryPath, content.join('\n'));
