import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { ApprovalMode } from "../config.js";

export interface IndPolicy {
  approval?: ApprovalMode;
  allowedProviders?: string[];
  allowedCommands?: string[];
  deniedCommands?: string[];
}

const POLICY_FILE = ".ind/policy.json";

function stringList(value: unknown, field: string): string[] | undefined {
  if (value === undefined) return undefined;
  if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || !item.trim())) throw new Error(`Invalid IND policy field '${field}': expected a list of non-empty strings.`);
  return value.map((item) => item.trim());
}

export function policyPath(projectRoot: string): string {
  return join(projectRoot, POLICY_FILE);
}

export function loadPolicy(projectRoot: string): IndPolicy {
  const path = policyPath(projectRoot);
  if (!existsSync(path)) return {};
  let parsed: unknown;
  try { parsed = JSON.parse(readFileSync(path, "utf8")); } catch (error) { throw new Error(`Invalid IND policy at ${path}: ${error instanceof Error ? error.message : String(error)}`); }
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) throw new Error(`Invalid IND policy at ${path}: expected a JSON object.`);
  const value = parsed as Record<string, unknown>;
  if (value.approval !== undefined && value.approval !== "chunk" && value.approval !== "command" && value.approval !== "never") throw new Error(`Invalid IND policy field 'approval': expected chunk, command, or never.`);
  const allowedProviders = stringList(value.allowedProviders, "allowedProviders");
  const allowedCommands = stringList(value.allowedCommands, "allowedCommands");
  const deniedCommands = stringList(value.deniedCommands, "deniedCommands");
  return {
    ...(value.approval ? { approval: value.approval as ApprovalMode } : {}),
    ...(allowedProviders ? { allowedProviders } : {}),
    ...(allowedCommands ? { allowedCommands } : {}),
    ...(deniedCommands ? { deniedCommands } : {}),
  };
}

function matches(command: string, patterns: string[]): boolean {
  return patterns.some((pattern) => {
    try { return new RegExp(pattern, "i").test(command); } catch (error) { throw new Error(`Invalid IND policy command pattern '${pattern}': ${error instanceof Error ? error.message : String(error)}`); }
  });
}

export function assertProviderAllowed(provider: string, policy: IndPolicy): void {
  if (policy.allowedProviders && !policy.allowedProviders.includes(provider)) throw new Error(`Provider '${provider}' is blocked by IND team policy.`);
}

export function assertPolicyCommandAllowed(command: string, policy: IndPolicy): void {
  if (policy.deniedCommands && matches(command, policy.deniedCommands)) throw new Error(`Command blocked by IND team policy: ${command}`);
  if (policy.allowedCommands && !matches(command, policy.allowedCommands)) throw new Error(`Command is not on the IND team policy allowlist: ${command}`);
}



