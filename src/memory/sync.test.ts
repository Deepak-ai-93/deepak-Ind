import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { assertSyncUrl, decryptMemory, encryptMemory } from "./sync.js";

describe("encrypted memory sync", () => {
  it("round-trips without putting plaintext in the envelope", () => {
    const content = "private decision: use local models";
    const envelope = encryptMemory(content, "a-long-test-secret");
    assert.equal(envelope.ciphertext.includes("private"), false);
    assert.equal(decryptMemory(envelope, "a-long-test-secret"), content);
    assert.throws(() => decryptMemory(envelope, "wrong-secret"));
  });

  it("requires HTTPS except for localhost testing", () => {
    assert.doesNotThrow(() => assertSyncUrl("https://sync.example.com/memory"));
    assert.doesNotThrow(() => assertSyncUrl("http://localhost:8080/memory"));
    assert.throws(() => assertSyncUrl("http://sync.example.com/memory"), /HTTPS/);
  });
});
