import { createCipheriv, createDecipheriv, randomBytes, scryptSync } from "node:crypto";
function keyFromSecret(secret) {
    if (secret.trim().length < 16)
        throw new Error("IND memory sync key must be at least 16 characters.");
    return scryptSync(secret, "ind-memory-sync-v1", 32);
}
export function encryptMemory(content, secret) {
    const iv = randomBytes(12);
    const cipher = createCipheriv("aes-256-gcm", keyFromSecret(secret), iv);
    const ciphertext = Buffer.concat([cipher.update(content, "utf8"), cipher.final()]);
    return { version: 1, algorithm: "aes-256-gcm", iv: iv.toString("base64"), authTag: cipher.getAuthTag().toString("base64"), ciphertext: ciphertext.toString("base64") };
}
export function decryptMemory(envelope, secret) {
    if (envelope.version !== 1 || envelope.algorithm !== "aes-256-gcm" || !envelope.iv || !envelope.authTag || !envelope.ciphertext)
        throw new Error("Unsupported or malformed IND memory sync envelope.");
    const decipher = createDecipheriv("aes-256-gcm", keyFromSecret(secret), Buffer.from(envelope.iv, "base64"));
    decipher.setAuthTag(Buffer.from(envelope.authTag, "base64"));
    return Buffer.concat([decipher.update(Buffer.from(envelope.ciphertext, "base64")), decipher.final()]).toString("utf8");
}
export function assertSyncUrl(value) {
    const url = new URL(value);
    if (url.protocol !== "https:" && !(url.protocol === "http:" && (url.hostname === "localhost" || url.hostname === "127.0.0.1")))
        throw new Error("IND memory sync requires HTTPS (HTTP is allowed only for localhost testing).");
    return url;
}
export async function pushEncryptedMemory(url, content, secret, fetcher = fetch) {
    const response = await fetcher(assertSyncUrl(url), { method: "PUT", headers: { "content-type": "application/json" }, body: JSON.stringify(encryptMemory(content, secret)) });
    if (!response.ok)
        throw new Error(`Memory sync push failed with HTTP ${response.status}.`);
}
export async function pullEncryptedMemory(url, secret, fetcher = fetch) {
    const response = await fetcher(assertSyncUrl(url), { headers: { accept: "application/json" } });
    if (!response.ok)
        throw new Error(`Memory sync pull failed with HTTP ${response.status}.`);
    return decryptMemory(await response.json(), secret);
}
//# sourceMappingURL=sync.js.map