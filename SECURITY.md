# Security Policy

Echo is local-first. The full threat model lives in `PLAN.md` section 10.

## Reporting a vulnerability

Email security reports to the maintainer listed at https://sarmalinux.com.
Do not open public issues for undisclosed vulnerabilities.

## Promises

- OAuth tokens live in the OS keychain, never on disk in plain text.
- Echo does not phone home. No analytics, no telemetry.
- Network egress happens only through skills the user has enabled.
