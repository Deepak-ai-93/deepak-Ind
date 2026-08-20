import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
const POLICY_FILE = ".ind/policy.json";
function stringList(value, field) {
    if (value === undefined)
        return undefined;
    if (!Array.isArray(value) || value.some((item) => typeof item !== "string" || !item.trim()))
        throw new Error(`Invalid IND policy field '${field}': expected a list of non-empty strings.`);
    return value.map((item) => item.trim());
}
export function policyPath(projectRoot) {
    return join(projectRoot, POLICY_FILE);
}
export function loadPolicy(projectRoot) {
    const path = policyPath(projectRoot);
    if (!existsSync(path))
        return {};
    let parsed;
    try {
        parsed = JSON.parse(readFileSync(path, "utf8"));
    }
    catch (error) {
        throw new Error(`Invalid IND policy at ${path}: ${error instanceof Error ? error.message : String(error)}`);
    }
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed))
        throw new Error(`Invalid IND policy at ${path}: expected a JSON object.`);
    const value = parsed;
    if (value.approval !== undefined && value.approval !== "chunk" && value.approval !== "command" && value.approval !== "never")
        throw new Error(`Invalid IND policy field 'approval': expected chunk, command, or never.`);
    const allowedProviders = stringList(value.allowedProviders, "allowedProviders");
    const allowedCommands = stringList(value.allowedCommands, "allowedCommands");
    const deniedCommands = stringList(value.deniedCommands, "deniedCommands");
    return {
        ...(value.approval ? { approval: value.approval } : {}),
        ...(allowedProviders ? { allowedProviders } : {}),
        ...(allowedCommands ? { allowedCommands } : {}),
        ...(deniedCommands ? { deniedCommands } : {}),
    };
}
function matches(command, patterns) {
    return patterns.some((pattern) => {
        try {
            return new RegExp(pattern, "i").test(command);
        }
        catch (error) {
            throw new Error(`Invalid IND policy command pattern '${pattern}': ${error instanceof Error ? error.message : String(error)}`);
        }
    });
}
export function assertProviderAllowed(provider, policy) {
    if (policy.allowedProviders && !policy.allowedProviders.includes(provider))
        throw new Error(`Provider '${provider}' is blocked by IND team policy.`);
}
export function assertPolicyCommandAllowed(command, policy) {
    if (policy.deniedCommands && matches(command, policy.deniedCommands))
        throw new Error(`Command blocked by IND team policy: ${command}`);
    if (policy.allowedCommands && !matches(command, policy.allowedCommands))
        throw new Error(`Command is not on the IND team policy allowlist: ${command}`);
}
//# sourceMappingURL=policy.js.map