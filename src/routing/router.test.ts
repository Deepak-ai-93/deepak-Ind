import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { classifyTask, routeTask } from "./router.js";
import type { IndConfig } from "../config.js";

const config: IndConfig = { projectRoot: ".", provider: "openai-compatible", model: "fallback", cheapModel: "cheap", strongModel: "strong", routing: "auto", approval: "chunk", maxInputTokens: 1000, maxOutputTokens: 100, maxToolTurns: 4 };

describe("model routing", () => {
  it("routes low-complexity work to the cheap model and code changes to the strong model", () => {
    assert.equal(classifyTask("summarize this file"), "summarize");
    assert.equal(routeTask("summarize this file", config).model, "cheap");
    assert.equal(routeTask("implement authentication", config).model, "strong");
  });
  it("falls back to the configured model when routing is disabled", () => {
    assert.equal(routeTask("implement authentication", { ...config, routing: "off" }).model, "fallback");
  });
});
