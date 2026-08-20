import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, isAbsolute, relative, resolve } from "node:path";

function safePath(root: string, filePath: string): string {
  const absolute = isAbsolute(filePath) ? resolve(filePath) : resolve(root, filePath);
  const fromRoot = relative(resolve(root), absolute);
  if (fromRoot.startsWith("..") || isAbsolute(fromRoot)) throw new Error(`Refusing path outside project: ${filePath}`);
  if (fromRoot === ".env" || fromRoot.startsWith(".env.") || fromRoot.startsWith(".git")) throw new Error(`Refusing sensitive path: ${filePath}`);
  return absolute;
}

export async function readProjectFile(root: string, filePath: string): Promise<string> {
  return readFile(safePath(root, filePath), "utf8");
}

export async function writeProjectFile(root: string, filePath: string, content: string, expectedContent?: string): Promise<string> {
  const absolute = safePath(root, filePath);
  if (expectedContent !== undefined) {
    const current = await readFile(absolute, "utf8").catch(() => undefined);
    if (current !== expectedContent) throw new Error(`Edit precondition failed for ${filePath}; file changed or does not exist.`);
  }
  await mkdir(dirname(absolute), { recursive: true });
  await writeFile(absolute, content, "utf8");
  return absolute;
}
