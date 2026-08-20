import { spawn, spawnSync } from "node:child_process";
const PACKAGE_NAME = "opencode-ai";
function executableName() {
    return process.platform === "win32" ? "opencode.cmd" : "opencode";
}
function canRun(command) {
    const result = spawnSync(command, ["--version"], { stdio: "ignore", shell: false });
    return !result.error && result.status === 0;
}
function install() {
    const npm = process.platform === "win32" ? "npm.cmd" : "npm";
    console.log(`OpenCode is not installed. Installing ${PACKAGE_NAME} globally...`);
    const result = spawnSync(npm, ["install", "--global", PACKAGE_NAME], { stdio: "inherit", shell: false });
    if (result.error)
        throw new Error(`Could not start npm: ${result.error.message}`);
    if (result.status !== 0)
        throw new Error(`OpenCode installation failed with exit code ${result.status ?? 1}.`);
}
export function launchOpenCode(args) {
    const command = executableName();
    if (!canRun(command)) {
        install();
        if (!canRun(command))
            throw new Error("OpenCode installed, but its executable is not on PATH. Restart the terminal and run 'ind opencode' again.");
    }
    const child = spawn(command, args, { stdio: "inherit", shell: false });
    child.once("error", (error) => { console.error(`OpenCode error: ${error.message}`); process.exitCode = 1; });
    child.once("exit", (code, signal) => { process.exitCode = signal ? 1 : (code ?? 1); });
}
//# sourceMappingURL=opencode.js.map