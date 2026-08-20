import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, isAbsolute, join, resolve } from "node:path";
const DEFAULTS = {
    provider: "openai-compatible",
    model: "",
    cheapModel: "",
    strongModel: "",
    routing: "auto",
    approval: "chunk",
    maxInputTokens: 12_000,
    maxOutputTokens: 4_000,
    maxToolTurns: 8,
};
function positiveInteger(value, fallback) {
    if (typeof value === "number" && Number.isInteger(value) && value > 0)
        return value;
    if (typeof value === "string" && /^\d+$/.test(value)) {
        const parsed = Number(value);
        if (parsed > 0)
            return parsed;
    }
    return fallback;
}
function stringValue(value, fallback) {
    return typeof value === "string" ? value.trim() : fallback;
}
function approvalValue(value) {
    return value === "chunk" || value === "command" || value === "never" ? value : DEFAULTS.approval;
}
function routingValue(value) {
    return value === "off" ? "off" : DEFAULTS.routing;
}
function readConfigFile(path) {
    if (!existsSync(path))
        return {};
    try {
        const parsed = JSON.parse(readFileSync(path, "utf8"));
        if (!parsed || typeof parsed !== "object" || Array.isArray(parsed))
            return {};
        return parsed;
    }
    catch (error) {
        throw new Error(`Invalid IND config at ${path}: ${error instanceof Error ? error.message : String(error)}`);
    }
}
function configPath(projectRoot) {
    const explicit = process.env.IND_CONFIG?.trim();
    if (!explicit)
        return join(projectRoot, ".ind", "config.json");
    return isAbsolute(explicit) ? explicit : resolve(projectRoot, explicit);
}
export function findProjectRoot(start = process.cwd()) {
    let current = resolve(start);
    while (true) {
        if (existsSync(join(current, ".git")) || existsSync(join(current, "package.json")))
            return current;
        const parent = dirname(current);
        if (parent === current)
            return resolve(start);
        current = parent;
    }
}
export function loadConfig(start = process.cwd()) {
    const projectRoot = findProjectRoot(start);
    const file = readConfigFile(configPath(projectRoot));
    const baseUrl = stringValue(file.baseUrl ?? process.env.IND_BASE_URL, "");
    const model = stringValue(file.model ?? process.env.IND_MODEL, DEFAULTS.model);
    const cheapModel = stringValue(file.cheapModel ?? process.env.IND_CHEAP_MODEL, DEFAULTS.cheapModel);
    const strongModel = stringValue(file.strongModel ?? process.env.IND_STRONG_MODEL, DEFAULTS.strongModel);
    return {
        projectRoot,
        provider: stringValue(file.provider ?? process.env.IND_PROVIDER, DEFAULTS.provider),
        model,
        cheapModel,
        strongModel,
        routing: routingValue(file.routing ?? process.env.IND_ROUTING),
        ...(baseUrl ? { baseUrl } : {}),
        approval: approvalValue(file.approval ?? process.env.IND_APPROVAL),
        maxInputTokens: positiveInteger(file.maxInputTokens ?? process.env.IND_MAX_INPUT_TOKENS, DEFAULTS.maxInputTokens),
        maxOutputTokens: positiveInteger(file.maxOutputTokens ?? process.env.IND_MAX_OUTPUT_TOKENS, DEFAULTS.maxOutputTokens),
        maxToolTurns: positiveInteger(file.maxToolTurns ?? process.env.IND_MAX_TOOL_TURNS, DEFAULTS.maxToolTurns),
    };
}
export function configFileLocation(projectRoot) {
    return configPath(projectRoot);
}
export function globalConfigDirectory() {
    return join(homedir(), ".ind");
}
//# sourceMappingURL=config.js.map