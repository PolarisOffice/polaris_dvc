# Security policy

polaris_dvc parses HWPX files — a ZIP-based container of
user-authored XML. Because documents are often received from
untrusted sources (email, downloads, ingest pipelines), parser
correctness has security implications. This document covers what
we consider a vulnerability, how to report one, and what to expect
after reporting.

## Supported versions

Pre-1.0 (`0.x`): only the latest `0.x` release on `main` receives
security fixes. Pin a specific version only if you accept that any
vulnerability found in older releases will be fixed forward, not
backported.

Once 1.0 is tagged, this section will be updated with the
supported-version window.

## Scope — what counts as a vulnerability

In scope:

- **Memory / resource safety in parsers.** Crashes, panics, infinite
  loops, unbounded allocation, stack overflow, or quadratic blow-up
  triggered by a malformed HWPX, ZIP container, or rule JSON.
- **Zip-slip / path traversal.** Any HWPX whose internal ZIP entries
  cause writes or reads outside the intended in-memory scope. (We
  don't write extracted files to disk in the library, but report it
  anyway — defense in depth.)
- **Information disclosure from the rule spec.** A malicious
  `spec.json` that causes the engine to leak unrelated file
  contents, environment variables, or network state.
- **WASM sandbox escapes** in the `polaris-dvc-wasm` crate or
  its web demo — e.g., a validator input that causes the wasm
  module to access host resources it shouldn't.
- **Integer overflow / under-read / under-write** in the OWPML
  parser (`polaris-dvc-hwpx`).

Out of scope:

- Correctness divergence from upstream hancom-io/dvc. File those
  as regular issues tagged `parity`.
- "This file is spec-invalid but we don't emit an error for it."
  Same — that's a `parity` or `feature` issue.
- Vulnerabilities in third-party dependencies that don't materially
  affect polaris usage. File those upstream; we track via
  Dependabot.
- Issues in the reference snapshot at `third_party/dvc-upstream/`.
  That's a read-only copy of external source; report to
  <https://github.com/hancom-io/dvc>.

## How to report

**Please do not open a public GitHub issue for security matters.**

Use **GitHub's private vulnerability reporting**:

1. Go to <https://github.com/PolarisOffice/polaris_dvc/security>.
2. Click **"Report a vulnerability"**.
3. Fill in the template. Attach a proof-of-concept HWPX / rule
   JSON if you have one.

This creates a private thread visible only to project maintainers
and the reporter. GitHub handles the CVE lifecycle if the report is
confirmed.

If GitHub's form is unavailable for some reason, open an issue
titled "security contact request" (no details in the body) and we
will reach out to arrange a secure channel.

## What to expect

- **Acknowledgement:** within 5 business days of receipt.
- **Initial triage:** within 14 days — we'll either accept the
  report with a rough severity estimate or explain why we don't
  consider it a vulnerability.
- **Fix timeline:** no fixed SLA pre-1.0, but typical turnaround
  for confirmed high-severity issues is 2–6 weeks. We'll keep the
  reporter informed.
- **Disclosure:** after a fix ships, we'll publish a GitHub Security
  Advisory with the CVE (if GitHub assigns one), a summary, and
  credit to the reporter unless they request anonymity.

## Safe-harbor

We will not pursue legal action against researchers who:

- Make a good-faith effort to follow this policy.
- Avoid privacy violations, data destruction, and service
  disruption to non-polaris systems during testing.
- Do not publicly disclose the vulnerability before a fix is
  released, unless we've agreed to a coordinated-disclosure date.

## Scope cross-reference

For the attack surface of each crate:

| crate | untrusted input? | notes |
|---|---|---|
| `polaris-dvc-hwpx` | yes (HWPX bytes) | primary parser — most exposure |
| `polaris-dvc-core` | yes (rule JSON, HWPX via hwpx crate) | secondary — processes outputs of hwpx |
| `polaris-dvc-format` | yes (bytes, before parser dispatch) | sniff + route |
| `polaris-dvc-cli` | yes (argv, stdin, files) | CLI flag parser + file I/O |
| `polaris-dvc-wasm` | yes (JS → WASM boundary) | browser / Node consumers |
