import {
  delay,
  waitFor,
} from 'https://deno.land/x/mighty_promise@v0.0.10/mod.ts';
const viteServer = 'http://localhost:3000';

const testFiles = [] as string[];

for await (const file of Deno.readDir('./deno')) {
  if (file.name.endsWith('test.ts')) {
    testFiles.push(file.name);
  }
}

if (testFiles.length === 0) {
  Deno.exit();
}

console.log('Core Number', navigator.hardwareConcurrency);
console.log();

let serverProcess;
if (!(await isServerAvailable())) {
  serverProcess = Deno.run({ cmd: ['pnpm', 'start'], stdout: 'null' });
  console.log('Starting server...');
  await waitFor({
    condition: isServerAvailable,
    timeout: 20_000,
  });
  await delay(1000);
  console.log('Starting server done\n');
}

console.log('Start testing...');
for (const file of testFiles) {
  const path = viteServer + '/deno/' + file;
  console.log('-----------------------------\n');
  console.log('>> ' + path + ':\n\n');
  Deno.run;
  await Deno.run({
    cmd: ['deno', 'test', '-Aq', '--location', viteServer, path, '--reload'],
    stdout: 'inherit',
  }).status();
  console.log();
}

const promise = serverProcess?.status();
serverProcess?.kill('SIGTERM');
serverProcess?.close();
await promise;

async function isServerAvailable() {
  const res = await fetch(viteServer);
  return res.status === 200;
}
