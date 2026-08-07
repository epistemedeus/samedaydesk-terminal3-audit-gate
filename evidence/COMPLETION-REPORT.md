# Terminal3 ADK bounty completion report

## Submission summary

SameDayDesk Audit Gate is an original Rust WASM contract deployed to the
Terminal3 testnet. It converts a private AI-search audit intake into a
deterministic public-safe remediation plan inside the TEE.

The work is based on Terminal3's official `z-tenant-flight` walkthrough fork,
but the use case, contract interface, scoring policy, tests, deployment script,
and evidence are original.

## Account and credits

- Agent ID / tenant DID:
  `did:t3n:9f5a67812cda76ea1601afb2b595d22fd4ccf142`
- Free test credits claimed: 20,000
- The one-time API key is stored outside the repository and is not included in
  any screenshot, log, commit, or report.
- Masked claim-success screenshot:
  [`terminal3-claim-success.png`](terminal3-claim-success.png)

## Contract deployment

- Contract ID: `484`
- Canonical script:
  `z:9f5a67812cda76ea1601afb2b595d22fd4ccf142:sdd-audit`
- Registered and resolved version: `0.1.0`
- WASM size: 149,166 bytes
- WASM SHA-256:
  `4f53e7bdc04fa4307c8e17d8faefe7d4363026058252c02c3ad5d4c898638918`

The stable machine-readable receipt is in
[`deployment.json`](deployment.json).

## Original use case

A SameDayDesk sales or delivery agent may know a prospect's domain, contact
email, internal notes, and monthly budget. Those four values should not appear
in public lead records or downstream agent responses.

`plan-audit` accepts the private values plus eight readiness signals. It
validates the input, calculates a readiness score and tier, and returns only:

- an opaque public id;
- a readiness score and tier;
- an ordered remediation plan;
- the count of withheld private fields;
- the policy version.

The contract declares no Terminal3 network, storage, logging, or tenant host
interface. Its deterministic business logic has no network or durable-storage
egress path.

## Real invocation result

```json
{
  "public_id": "terminal3-demo-001",
  "readiness_score": 48,
  "readiness_tier": "weak",
  "priority_actions": [
    "Publish Organization JSON-LD with a stable legal and brand identity",
    "Connect named experts, credentials, and reviewed services in structured data",
    "Reconcile location names, addresses, and phone data across owned profiles",
    "Add verifiable third-party authority evidence and cite its original source",
    "Add concise answer-shaped FAQs with matching visible page content",
    "Publish a small llms.txt index after the primary entity gaps are fixed"
  ],
  "private_fields_withheld": 4,
  "policy_version": "0.1.0"
}
```

The deployment script searched the serialized response for every exact
synthetic private value: business domain, contact email, internal notes, and
budget. All four were absent.

## Walkthrough followed

1. Signed up for the ADK and claimed the free test credits.
2. Forked Terminal3's official Rust example.
3. Replaced the flight demo with an original SameDayDesk privacy use case.
4. Added strict deserialization, deterministic scoring, and eight native tests.
5. Built a `wasm32-wasip2` component and verified it with `wasm-tools`.
6. Authenticated with the one-time key using the official SDK.
7. Registered version `0.1.0`, resolved it from the live node, and invoked it.
8. Ran a four-value response leak check and preserved public-only evidence.

## Verification

- Rust tests: 8 passed, 0 failed
- Formatting: passed
- Clippy with warnings denied: passed
- WASM build: passed
- `wasm-tools validate`: passed
- Testnet authentication: passed
- Testnet registration: passed
- Real testnet invocation: passed
- Private-value leak check: passed

Full details are in [`TEST-RESULTS.md`](TEST-RESULTS.md).

## Bugs and recommendations

The walkthrough found four actionable issues:

1. the SDK's signed testnet trust-manifest endpoint returns HTTP 405;
2. SDK 4.30.0 and the live node disagree on the `token.get-usage` wire shape;
3. the SDK dependency tree carries a critical archive-extraction advisory;
4. the one-line minified SDK bundle overwhelms Node error output.

Reproduction steps, impact, and workarounds are in
[`BUGS.md`](BUGS.md).

## Evidence index

- Source and instructions: repository root `README.md`
- Contract implementation: `src/audit.rs` and `src/lib.rs`
- Interface: `wit/world.wit`
- Reproducible deployer: `scripts/deploy.mjs`
- Stable live receipt: `evidence/deployment.json`
- Verification record: `evidence/TEST-RESULTS.md`
- Bug report: `evidence/BUGS.md`
- Masked credit-claim proof: `evidence/terminal3-claim-success.png`
