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
console.log('Start testing...');
for (const file of testFiles) {
  const path = 'http://localhost:3000/deno/' + file;
  console.log('-----------------------------\n');
  console.log('>> ' + path + ':\n\n');
  Deno.run;
  await Deno.run({
    cmd: [
      'deno',
      'test',
      '-Aq',
      '--location',
      'http://localhost:3000/',
      path,
      '--reload',
    ],
    stdout: 'inherit',
  }).status();
  console.log();
}
