export function formatUsageSummary(summary) {
    const savings = summary.baselineInputTokens > 0 ? `${summary.tokensSaved.toLocaleString()} saved vs ${summary.baselineInputTokens.toLocaleString()} baseline` : "baseline not recorded";
    return [
        `sessions: ${summary.sessions}  events: ${summary.events}`,
        `tokens: ${summary.inputTokens.toLocaleString()} in / ${summary.outputTokens.toLocaleString()} out / ${summary.totalTokens.toLocaleString()} total`,
        `cached: ${summary.cachedTokens.toLocaleString()}  cost: $${summary.estimatedCost.toFixed(6)}  avg latency: ${Math.round(summary.averageLatencyMs)}ms`,
        `savings: ${savings}`,
    ].join("\n");
}
export function renderLiveUsage(summary, output = process.stdout) {
    const line = `IND usage | ${summary.inputTokens} in + ${summary.outputTokens} out | $${summary.estimatedCost.toFixed(6)} | ${Math.round(summary.averageLatencyMs)}ms`;
    if (output.isTTY)
        output.write(`\r${line}`);
    else
        output.write(`${line}\n`);
}
//# sourceMappingURL=monitor.js.map