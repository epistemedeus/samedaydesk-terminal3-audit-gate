# Compatibility review, 2026-09-22

No contract rebuild, version bump, or same-slot update is required.
Contract 484 stays on version `0.1.0`.

This review checked the provider notice of 2026-09-17 (per-function grants,
and a possible `host:tenant` 1.2 removal, may require a major
`CONTRACT_VERSION` and an update of the same contract slot) against the
current Terminal 3 docs, `@terminal3/t3n-sdk` 5.19.0 and 5.20.0, and the
imports of the component this repository actually builds.

## What the current platform says

Confirmed from the live docs and the published SDK on 2026-09-22:

- [Changelog](https://docs.terminal3.io/developers/adk/changelog) records two
  relevant testnet changes. `ap2` (`ap2/mandate`) was removed in
  `testnet-v1.0.10` (2026-09-15): a component that still imports it fails to
  instantiate. Delegation grants became one function per row in
  `testnet-v1.0.11` (2026-09-18), shipping in `@terminal3/t3n-sdk` 5.19.0
  (published 2026-09-17). The old `functions: string[]` shape is rejected.
  The changelog does not say `host:tenant/tenant-context@1.2.0` was removed.
- [Host API](https://docs.terminal3.io/t3n/how-t3n-works/host-api) and the
  [SDK reference](https://docs.terminal3.io/developers/adk/reference) mark
  `ap2/mandate` removed and still list `tenant` (`tenant-context`) as
  available.
- [Write your TEE contract](https://docs.terminal3.io/developers/adk/get-started/walkthrough/write-contract)
  still tells a contract that needs tenant context to vendor
  `host-tenant-1.2.0` and import `host:tenant/tenant-context@1.2.0`.
- [Member delegation](https://docs.terminal3.io/developers/adk/get-started/member-delegation)
  is a client policy document, not a guest WIT change. A missing grant fails
  when a contract reaches the network (`host/http.egress_denied`), not at
  function entry.
- SDK 5.20.0 (published 2026-09-22) has the same public `dist/index.d.ts` as
  5.19.0. `TenantContractsNamespace.register` / `publish` take
  `{ tail, version, wasm }`. They do not take a contract id. Current docs
  still say registering again at the same tail allocates a new `contract_id`.

## What this contract imports

`wit/world.wit` has no `import`. The vendored `host:tenant@1.0.0`,
`host:interfaces@2.1.0`, and `host:outbox@1.0.0` packages are not referenced
by the world. `wasm-tools component wit` on the local `wasm32-wasip2`
release build shows:

- imports: `wasi:io`, `wasi:clocks`, and `wasi:cli` at `0.2.9` only (the Rust
  WASI CLI adapter already described in the README; the contract code does
  not call it)
- export: `z:samedaydesk-audit/contracts@0.1.0` with `plan-audit`
- no `ap2/mandate`, `host:tenant` (1.0.0 or 1.2.0), `host:interfaces`, or
  `host:outbox`

The upstream flight example imports `host:tenant/tenant-context@1.0.0` and
HTTP. This contract does not. The removed AP2 interface is not assumed.

## Applicability

| Notice | Applies here? | Why |
| --- | --- | --- |
| `ap2/mandate` removed | No | The component does not import it. |
| One function per delegation grant | No guest change | This repo writes no grant. The only export is `plan-audit`, and it does not import HTTP, so there is no egress grant to rewrite. |
| `host:tenant` 1.2 removal | No | Docs still publish `@1.2.0` for contracts that use it. This component imports no `host:tenant` version. |
| Major `CONTRACT_VERSION` and same-slot update | No | Not triggered. A major bump would change public `policy_version`. The public SDK has no same-slot wasm replace. |

`CONTRACT_VERSION`, the WIT package, `scripts/deploy.mjs`, and `package.json`
stay at `0.1.0`. Tail stays `sdd-audit`. Scoring, validation, and the
four-field withholding result are unchanged.

## Live identity to keep

From `evidence/deployment.json` (2026-08-07). This review did not register
or invoke the contract.

- Environment: testnet
- Tenant: `did:t3n:9f5a67812cda76ea1601afb2b595d22fd4ccf142`
- Contract id: `484`
- Script: `z:9f5a67812cda76ea1601afb2b595d22fd4ccf142:sdd-audit`
- Version: `0.1.0`
- Deployed wasm: 149,166 bytes,
  SHA-256 `4f53e7bdc04fa4307c8e17d8faefe7d4363026058252c02c3ad5d4c898638918`

A local rebuild with rustc 1.98.1 produces different bytes because the
compiler changed. Those bytes were not deployed and must not be registered
over slot 484.

## Migration

Do not run a migration.

The testnet key, if Root later invokes the existing contract, is the
environment variable `T3N_API_KEY`, kept outside this repository. Do not
print it and do not commit it. This review did not read it.

```sh
# Do not run. register/publish is not a same-slot update: a higher version
# at tail sdd-audit allocates a new contract_id under current docs.
# T3N_API_KEY is taken from the operator environment only.
# T3N_ALLOW_UNSAFE_TRUST must not be set for this review.
# npm run deploy:testnet
```

There is no public command that replaces the wasm in contract id 484.

## Rollback and version

No on-chain write was made, so there is nothing to roll back. Required
version remains `0.1.0` on contract id 484.

If a later registration at a higher version happens outside this review,
current docs say unpinned calls follow the latest version and an explicit
`contract_version` / `script_version` of `0.1.0` still resolves the original
build. That pin does not undo a newly allocated `contract_id`. This review
does not create one.
