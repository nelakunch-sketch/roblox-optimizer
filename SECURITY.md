# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest release | ✅ |
| Older releases  | ❌ Please upgrade |

## Reporting a Vulnerability

If you find a security issue (e.g., the optimizer could be abused to escalate privileges or harm systems), please **do not open a public issue**.

Instead, email: `security@yourdomain.com`  
Or use GitHub's private [Security Advisories](../../security/advisories/new).

We aim to respond within **72 hours** and release a patch within **7 days** for confirmed critical issues.

## Scope

This project uses elevated Windows APIs by design (Admin-required). Reports are in scope if they describe:

- Privilege escalation beyond what the user explicitly granted
- Silent persistence / startup modification without user consent
- Code execution from untrusted input
- Supply-chain issues in dependencies

Out of scope: "the program requires Admin" — that is intentional and documented.
