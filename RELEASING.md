# Releasing IND

Replace `<owner>/<repo>` with the GitHub repository before publishing.

## GitHub installation

```bash
npm install -g github:<owner>/<repo>
ind --help
```

## npm release

The package is currently not published on npm. A maintainer must authenticate with npm before running `npm publish`.

Publishing to npm is the recommended way for users to install IND globally on any supported device. Vercel is not required for CLI distribution; use it only for an optional website or hosted dashboard.

```bash
npm login
npm version patch
npm publish
```

The `prepublishOnly` script runs typecheck, tests, and the production build. Verify the package contents with `npm pack --dry-run` before publishing.

## GitHub setup

1. Create a public repository.
2. Push the source repository and default branch.
3. Enable Issues and private security reporting.
4. Add the repository URL to `package.json`.
5. Create a GitHub release matching the npm version.


