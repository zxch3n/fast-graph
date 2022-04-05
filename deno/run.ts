import { cac } from "https://unpkg.com/cac@6.7.12/mod.ts";
import {
  delay,
  waitFor,
} from "https://deno.land/x/mighty_promise@v0.0.10/mod.ts";

const cli = cac("test-runner");
cli.command("test", "run deno tests").action(runTests);
cli.command("bench", "benchmarking").action(runBenches);
cli.help();
cli.parse();

const viteServer = "http://localhost:3000";

async function runBenches() {
  const testFiles = [] as string[];

  for await (const file of Deno.readDir("./deno")) {
    if (file.name.endsWith(".bench.ts")) {
      testFiles.push(file.name);
    }
  }
  if (testFiles.length === 0) {
    Deno.exit();
  }

  runWithServerStart(async () => {
    console.log("Core Number", navigator.hardwareConcurrency);
    console.log();

    console.log("Start benching...");
    console.log("-----------------------------\n");
    for (const file of testFiles) {
      const path = viteServer + "/deno/" + file;
      console.log(">> " + file + ":\n");
      await Deno.run({
        cmd: [
          "deno",
          "run",
          "--reload",
          "-Aq",
          "--location",
          viteServer,
          path,
        ],
        stdout: "inherit",
      }).status();
      console.log("\n");
    }

    console.log();
  });
}

async function runTests() {
  const testFiles = [] as string[];

  for await (const file of Deno.readDir("./deno")) {
    if (file.name.endsWith("test.ts")) {
      testFiles.push(file.name);
    }
  }
  if (testFiles.length === 0) {
    Deno.exit();
  }

  runWithServerStart(async () => {
    console.log("Core Number", navigator.hardwareConcurrency);
    console.log();

    console.log("Start testing...");
    const path = testFiles.map((file) => viteServer + "/deno/" + file);
    console.log("-----------------------------\n");
    console.log(">> " + testFiles.join(", ") + ":\n\n");
    await Deno.run({
      cmd: [
        "deno",
        "test",
        "--reload",
        "-Aq",
        "--location",
        viteServer,
        ...path,
      ],
      stdout: "inherit",
    }).status();
    console.log();
  });
}

async function runWithServerStart(fn: () => void | Promise<void>) {
  let serverProcess;
  if (!(await isServerAvailable())) {
    serverProcess = Deno.run({ cmd: ["pnpm", "start"], stdout: "null" });
    console.log("Starting server...");
    await waitFor({
      condition: isServerAvailable,
      timeout: 20_000,
    });
    await delay(1000);
    console.log("Starting server done\n");
  }

  try {
    await fn();
  } finally {
    const promise = serverProcess?.status();
    serverProcess?.kill("SIGTERM");
    serverProcess?.close();
    await promise;
  }
}

async function isServerAvailable() {
  const res = await fetch(viteServer);
  return res.status === 200;
}
