# Terminal3 walkthrough findings

These findings came from the pinned `@terminal3/t3n-sdk` 4.30.0 package and
the live Terminal3 testnet on 2026-08-07. No API key or private prospect value
is included below.

## 1. Signed testnet trust manifest returns HTTP 405

Severity: high for secure developer onboarding.

### Reproduction

```js
setEnvironment("testnet");
await fetchTrustedManifest("testnet");
```

Observed result:

```text
Trust manifest request to
https://cn-api.sg.testnet.t3n.terminal3.io/api/trust-manifest failed: 405
```

The same SDK declares `trustAnchor` as a required `T3nClient` option. Omitting
it causes a `TypeError` while reading `unsafe_trust_server`, rather than a
targeted validation error. The signed manifest path therefore cannot currently
satisfy the required secure configuration on testnet.

The public
[Agent Developer Kit page](https://terminal3.io/products/agent-developer-kit)
currently shows a quickstart that constructs `T3nClient` without
`trustAnchor`. That visible snippet fails with the pinned 4.30.0 SDK before the
handshake begins.

Expected: the signed manifest endpoint returns a valid operator-signed trust
anchor, or the SDK reports an actionable availability error before client
construction.

Testnet-only workaround used here: attempt the signed manifest first, then
require the explicit environment flag `T3N_ALLOW_UNSAFE_TRUST=1` before using
`{ unsafe_trust_server: true }`. The script has no silent fallback, and the
workaround must never be used in production.

## 2. `token.get-usage` SDK and testnet wire shapes disagree

Severity: medium; credit visibility is broken while core deployment works.

### Reproduction

After a successful handshake and Ethereum-key authentication:

```js
await t3n.getUsage();
```

Observed result, with the encrypted payload and request id redacted:

```text
RPC Error: invalid token.get-usage params: invalid type: encrypted string
[redacted], expected struct GetUsageParams
```

The SDK type documentation says this self-only read is sent as a sealed
session payload. The live testnet rejects that sealed string and expects a
plain parameter struct. Authentication, tenant lookup, contract registration,
version resolution, and invocation all succeeded in the same session.

The same public ADK quickstart calls `client.getUsage()` immediately after
authentication, so the documented happy path reaches this mismatch directly.

Expected: SDK 4.30.0 and the live node agree on the parameter encoding, and
`getUsage()` returns the documented `UsagePage`.

Workaround: treat the balance read as optional evidence. The 20,000-credit
claim is visible on the signup success screen; deployment and invocation do
not depend on `getUsage()`.

## 3. SDK install carries an archive-extraction vulnerability

Severity: critical according to `npm audit`; exploitability depends on whether
the affected build-time archive path processes attacker-controlled content.

### Reproduction

```bash
npm install
npm audit --omit=dev
```

Observed dependency path:

```text
@terminal3/t3n-sdk
  -> @bytecodealliance/jco
    -> @bytecodealliance/componentize-js
      -> @bytecodealliance/weval
        -> decompress
```

`npm audit` reports four findings: three moderate and one critical. The
critical `decompress` advisories cover archive path traversal / arbitrary file
write outside the extraction directory.

This repository's deployment path loads a prebuilt WASM component and does not
extract an untrusted archive, so the known path was not exercised here.
Terminal3 should nevertheless update or override the vulnerable transitive
dependency and publish a patched SDK release.

## 4. Minified distribution makes runtime failures extremely noisy

Severity: low developer-experience issue.

The ESM distribution is emitted as a single multi-megabyte line. A normal Node
stack trace prints that entire line before the actual exception, overwhelming
CI logs and hiding the actionable message. Source maps or line-preserving
minification would make the SDK substantially easier to debug.
