# Security policy

## Reporting a vulnerability

Please do not open a public issue. Report privately through GitHub: open the repository's Security tab and choose "Report a vulnerability". Include steps to reproduce, the version, and your OS. You can expect a first reply within a few days.

## Scope and design

- Speech to text runs locally. Audio, history and settings stay in the app data folder.
- Network use: model downloads, update checks, and cleanup only if a cloud provider is configured by the user.
- API keys for cleanup providers are stored in the settings file on disk.
- The webview runs with a restrictive content security policy and a narrow file scope.

## Supported versions

Only the latest release is supported while the project is pre-1.0.
