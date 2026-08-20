export interface BenchmarkCase {
  id: string;
  task: string;
  root: string;
  expectedFiles: string[];
  budgetTokens: number;
}

export interface BenchmarkResult {
  id: string;
  task: string;
  baselineInputTokens: number;
  selectedInputTokens: number;
  tokensSaved: number;
  savingsPercent: number;
  expectedFiles: string[];
  selectedFiles: string[];
  expectedFilesSelected: number;
  relevanceRecall: number;
  underBudget: boolean;
}
