import { randomUUID } from "node:crypto";
import type { TaskPlan } from "./types.js";

export function createTaskPlan(task: string, contextFiles: string[] = []): TaskPlan {
  const normalized = task.trim();
  if (!normalized) throw new Error("Task description cannot be empty.");
  const id = randomUUID();
  return {
    id,
    task: normalized,
    contextFiles: [...new Set(contextFiles)],
    createdAt: new Date().toISOString(),
    chunks: [
      { id: `${id}-01`, sequence: 1, title: "Understand the change", goal: `Confirm the smallest change needed for: ${normalized}`, status: "pending", verification: ["Context is selected and the task scope is explicit."] },
      { id: `${id}-02`, sequence: 2, title: "Implement the change", goal: `Apply the approved code change for: ${normalized}`, status: "pending", verification: ["The requested files are changed and the diff is reviewable."] },
      { id: `${id}-03`, sequence: 3, title: "Verify the change", goal: "Run the configured verification command and report the result.", status: "pending", verification: ["The verification command exits successfully."] },
    ],
  };
}
