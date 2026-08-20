import { spawn } from "node:child_process";
import { assertPolicyCommandAllowed, loadPolicy } from "../policy/policy.js";
const MAX_OUTPUT_BYTES = 32_000;
const BLOCKED_PATTERNS = [/\brm\s+-rf\b/i, /\bdel\s+\/s\b/i, /\bformat\s+[a-z]:/i, /\bshutdown\b/i, /\breg\s+delete\b/i];
export function assertCommandAllowed(command) {
    if (!command.trim())
        throw new Error("Command cannot be empty.");
    if (BLOCKED_PATTERNS.some((pattern) => pattern.test(command)))
        throw new Error(`Command blocked by IND safety policy: ${command}`);
}
export async function runProjectCommand(command, cwd, options = {}) {
    assertCommandAllowed(command);
    assertPolicyCommandAllowed(command, options.policy ?? loadPolicy(cwd));
    if (!options.approved)
        throw new Error("Command requires explicit approval.");
    const timeoutMs = options.timeoutMs ?? 120_000;
    return new Promise((resolve, reject) => {
        const child = spawn(command, { cwd, shell: true, windowsHide: true });
        let stdout = "";
        let stderr = "";
        let timedOut = false;
        const append = (current, chunk) => `${current}${chunk.toString("utf8")}`.slice(-MAX_OUTPUT_BYTES);
        const timer = setTimeout(() => { timedOut = true; child.kill(); }, timeoutMs);
        child.stdout.on("data", (chunk) => { stdout = append(stdout, chunk); });
        child.stderr.on("data", (chunk) => { stderr = append(stderr, chunk); });
        child.on("error", (error) => { clearTimeout(timer); reject(error); });
        child.on("close", (exitCode) => { clearTimeout(timer); resolve({ command, exitCode, stdout, stderr, timedOut }); });
    });
}
//# sourceMappingURL=commands.js.map