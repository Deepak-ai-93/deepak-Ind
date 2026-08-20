export class ProviderError extends Error {
    status;
    provider;
    retryable;
    constructor(provider, message, options = {}) {
        super(message);
        this.name = "ProviderError";
        this.provider = provider;
        this.status = options.status;
        this.retryable = options.retryable ?? false;
    }
}
//# sourceMappingURL=types.js.map