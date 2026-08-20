export type ChunkStatus = "pending" | "waiting-approval" | "running" | "passed" | "failed" | "blocked";

export interface TaskChunk {
  id: string;
  sequence: number;
  title: string;
  goal: string;
  status: ChunkStatus;
  verification: string[];
}

export interface TaskPlan {
  id: string;
  task: string;
  contextFiles: string[];
  chunks: TaskChunk[];
  createdAt: string;
}

export interface FileEditAction {
  type: "edit";
  path: string;
  content: string;
  expectedContent?: string;
}

export interface CommandAction {
  type: "command";
  command: string;
  timeoutMs?: number;
}

export type ChunkAction = FileEditAction | CommandAction;

export interface TaskRunEvent {
  type: "chunk-start" | "approval-required" | "edit-applied" | "command-finished" | "chunk-passed" | "chunk-failed" | "plan-complete";
  chunkId?: string;
  message: string;
}
