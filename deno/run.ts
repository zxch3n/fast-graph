import * as path from "https://deno.land/std@0.105.0/path/mod.ts";
import { cac } from "https://unpkg.com/cac@6.7.12/mod.ts";

export function getDirName(meta: ImportMeta) {
  const iURL = meta.url,
    fileStartRegex = /(^(file:)((\/\/)?))/,
    __dirname = path
      .join(iURL, "../")
      .replace(fileStartRegex, "")
      .replace(/(\/$)/, ""),
    __filename = iURL.replace(fileStartRegex, "");

  return { __dirname, __filename };
}

const cli = cac("test-runner");
cli.command("test", "run deno tests").action(runTests);
cli.command("bench", "benchmarking").action(runBenches);
cli.help();
cli.parse();

async function runBenches() {
  await Deno.run({
    cmd: ["deno", "bench", "--unstable", "-A"],
    stdout: "inherit",
    cwd: getDirName(import.meta).__dirname,
  }).status();
}

async function runTests() {
  await Deno.run({
    cmd: ["deno", "test", "-A"],
    stdout: "inherit",
    cwd: getDirName(import.meta).__dirname,
  }).status();
}
