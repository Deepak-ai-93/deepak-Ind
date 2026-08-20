import { runProjectCommand } from "../tools/commands.js";
import { writeProjectFile } from "../tools/files.js";
export async function runTaskPlan(plan, actions, options, onEvent) {
    const emit = (event) => onEvent?.(event);
    for (const chunk of plan.chunks) {
        chunk.status = "waiting-approval";
        emit({ type: "approval-required", chunkId: chunk.id, message: `Approval required for chunk ${chunk.sequence}: ${chunk.title}` });
        if (!options.approvedChunks.has(chunk.id)) {
            chunk.status = "blocked";
            throw new Error(`Chunk ${chunk.id} is blocked until approved.`);
        }
        chunk.status = "running";
        emit({ type: "chunk-start", chunkId: chunk.id, message: `Starting chunk ${chunk.sequence}: ${chunk.title}` });
        try {
            for (const action of actions.get(chunk.id) ?? []) {
                if (action.type === "edit") {
                    await writeProjectFile(options.projectRoot, action.path, action.content, action.expectedContent);
                    emit({ type: "edit-applied", chunkId: chunk.id, message: `Applied edit: ${action.path}` });
                }
                else {
                    const result = await runProjectCommand(action.command, options.projectRoot, { ...(action.timeoutMs === undefined ? {} : { timeoutMs: action.timeoutMs }), approved: true });
                    emit({ type: "command-finished", chunkId: chunk.id, message: `${action.command} exited ${result.exitCode ?? "unknown"}` });
                    if (result.timedOut || result.exitCode !== 0)
                        throw new Error(`Verification failed for '${action.command}': ${result.stderr || result.stdout}`);
                }
            }
            chunk.status = "passed";
            emit({ type: "chunk-passed", chunkId: chunk.id, message: `Chunk ${chunk.sequence} passed.` });
        }
        catch (error) {
            chunk.status = "failed";
            emit({ type: "chunk-failed", chunkId: chunk.id, message: error instanceof Error ? error.message : String(error) });
            throw error;
        }
    }
    emit({ type: "plan-complete", message: `Task plan ${plan.id} completed.` });
    return plan;
}
//# sourceMappingURL=runner.js.map