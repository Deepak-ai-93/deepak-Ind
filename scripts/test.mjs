import { readdirSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { spawnSync } from "node:child_process";

const root = resolve("src");
const files = [];

function visit(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) visit(path);
    else if (entry.isFile() && entry.name.endsWith(".test.ts")) files.push(relative(process.cwd(), path));
  }
}

visit(root);
files.sort();
if (files.length === 0) throw new Error("No TypeScript test files found.");

for (const file of files) {
  console.log(`\n[test] ${file}`);
  const result = spawnSync(process.execPath, ["--import", "tsx", file], { stdio: "inherit" });
  if (result.error) throw result.error;
  if ((result.status ?? 1) !== 0) process.exit(result.status ?? 1);
}
console.log(`\nPassed ${files.length} test files.`);



