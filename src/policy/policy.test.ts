import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { assertPolicyCommandAllowed, assertProviderAllowed } from "./policy.js";

describe("team policy", () => {
  it("enforces provider and command restrictions", () => {
    const policy = { allowedProviders: ["openai-compatible"], allowedCommands: ["npm test"], deniedCommands: ["npm publish"] };
    assert.doesNotThrow(() => assertProviderAllowed("openai-compatible", policy));
    assert.throws(() => assertProviderAllowed("anthropic", policy), /blocked/);
    assert.doesNotThrow(() => assertPolicyCommandAllowed("npm test", policy));
    assert.throws(() => assertPolicyCommandAllowed("npm publish", policy), /blocked/);
    assert.throws(() => assertPolicyCommandAllowed("npm run build", policy), /allowlist/);
  });
});
