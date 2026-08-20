import { readdir, readFile, stat } from "node:fs/promises";
import { join, relative, sep } from "node:path";

export interface RepositoryFile {
  relativePath: string;
  absolutePath: string;
  extension: string;
  bytes: number;
  modifiedAtMs: number;
  isSource: boolean;
}

export interface RepositorySnapshot {
  root: string;
  files: RepositoryFile[];
  ignoredDirectories: string[];
  scannedAt: string;
}

const IGNORED_DIRECTORIES = new Set([
  ".git", ".hg", ".svn", "node_modules", "dist", "build", "coverage", ".ind", ".agents", "output", ".next", "target", "vendor",
]);

const SOURCE_EXTENSIONS = new Set([
  ".c", ".cc", ".cpp", ".css", ".go", ".h", ".hpp", ".html", ".java", ".js", ".json", ".jsx", ".md", ".php", ".py", ".rb", ".rs", ".sh", ".sql", ".svelte", ".toml", ".ts", ".tsx", ".vue", ".yaml", ".yml",
]);

function normalizePath(path: string): string {
  return path.split(sep).join("/");
}

async function walk(root: string, current: string, files: RepositoryFile[], ignored: Set<string>): Promise<void> {
  const entries = await readdir(current, { withFileTypes: true });
  for (const entry of entries) {
    const absolutePath = join(current, entry.name);
    const relativePath = normalizePath(relative(root, absolutePath));
    if (entry.isDirectory()) {
      if (IGNORED_DIRECTORIES.has(entry.name)) {
        ignored.add(relativePath);
        continue;
      }
      await walk(root, absolutePath, files, ignored);
      continue;
    }
    if (!entry.isFile() || entry.name.startsWith(".") && entry.name !== ".env.example") continue;
    const metadata = await stat(absolutePath);
    const extension = entry.name.includes(".") ? `.${entry.name.split(".").pop()?.toLowerCase() ?? ""}` : "";
    files.push({ relativePath, absolutePath, extension, bytes: metadata.size, modifiedAtMs: metadata.mtimeMs, isSource: SOURCE_EXTENSIONS.has(extension) });
  }
}

export async function inspectRepository(root: string): Promise<RepositorySnapshot> {
  const files: RepositoryFile[] = [];
  const ignored = new Set<string>();
  await walk(root, root, files, ignored);
  files.sort((left, right) => left.relativePath.localeCompare(right.relativePath));
  return { root, files, ignoredDirectories: [...ignored].sort(), scannedAt: new Date().toISOString() };
}

export async function readRepositoryFile(file: RepositoryFile): Promise<string | undefined> {
  if (!file.isSource || file.bytes > 200_000) return undefined;
  try {
    const content = await readFile(file.absolutePath, "utf8");
    if (content.includes("\u0000")) return undefined;
    return content;
  } catch {
    return undefined;
  }
}
