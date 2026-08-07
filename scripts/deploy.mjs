import {
  T3nClient,
  TenantClient,
  createEthAuthInput,
  eth_get_address,
  fetchTrustedManifest,
  getNodeUrl,
  getScriptVersion,
  loadWasmComponent,
  metamask_sign,
  setEnvironment,
} from "@terminal3/t3n-sdk";
import { readFile } from "node:fs/promises";

const key = process.env.T3N_API_KEY;
if (!key || !/^0x[0-9a-fA-F]{64}$/.test(key)) {
  throw new Error("T3N_API_KEY is missing or has an invalid shape");
}

setEnvironment("testnet");

const wasmComponent = await loadWasmComponent();
let trustAnchor;
let trustMode = "operator-signed-manifest";
try {
  trustAnchor = await fetchTrustedManifest("testnet");
} catch (error) {
  if (process.env.T3N_ALLOW_UNSAFE_TRUST !== "1") {
    throw new Error(
      "The signed testnet trust manifest is unavailable. Set " +
        "T3N_ALLOW_UNSAFE_TRUST=1 only for an explicit testnet-only opt-out.",
      { cause: error },
    );
  }
  trustAnchor = { unsafe_trust_server: true };
  trustMode = "explicit testnet-only opt-out; signed manifest unavailable";
}
const address = eth_get_address(key);
const t3n = new T3nClient({
  wasmComponent,
  trustAnchor,
  handlers: {
    EthSign: metamask_sign(address, undefined, key),
  },
});

await t3n.handshake();
const authenticated = await t3n.authenticate(createEthAuthInput(address));
const tenantDid = authenticated.value;
const tenant = new TenantClient({
  t3n,
  baseUrl: getNodeUrl(),
  tenantDid,
});

await tenant.tenant.me();
const usageFindings = [];
async function readAvailableCredits(stage) {
  try {
    return (await t3n.getUsage()).balance.available;
  } catch (error) {
    const normalized = String(error?.message ?? error)
      .replace(/string "[^"]+"/, "encrypted string [redacted]")
      .replace(/ \[[0-9a-f-]{36}\]$/, "");
    usageFindings.push({
      stage,
      error: normalized,
    });
    return null;
  }
}
const creditsBefore = await readAvailableCredits("before deployment");

const wasm = await readFile(
  "target/wasm32-wasip2/release/samedaydesk_audit_gate.wasm",
);
const tail = "sdd-audit";
const contractVersion = "0.1.0";
const registered = await tenant.contracts.register({
  tail,
  version: contractVersion,
  wasm,
});

const tenantId = tenantDid.slice("did:t3n:".length);
const scriptName = `z:${tenantId}:${tail}`;
const scriptVersion = await getScriptVersion(getNodeUrl(), scriptName);

const syntheticIntake = {
  public_id: "terminal3-demo-001",
  business_domain: "private-prospect.example",
  contact_email: "owner@private-prospect.example",
  internal_notes: "Synthetic confidential note used only for the leak test",
  monthly_budget_cents: 75000,
  signals: {
    has_organization_schema: false,
    has_local_business_schema: true,
    has_person_authority_schema: false,
    has_faq_schema: false,
    has_llms_txt: false,
    has_indexable_service_pages: true,
    authority_evidence_count: 2,
    location_consistency_pct: 70,
  },
};

const output = await t3n.executeAndDecode({
  script_name: scriptName,
  script_version: scriptVersion,
  function_name: "plan-audit",
  input: syntheticIntake,
});

const rendered = JSON.stringify(output);
for (const forbidden of [
  syntheticIntake.business_domain,
  syntheticIntake.contact_email,
  syntheticIntake.internal_notes,
  String(syntheticIntake.monthly_budget_cents),
]) {
  if (rendered.includes(forbidden)) {
    throw new Error(`private-field leak detected for synthetic marker: ${forbidden}`);
  }
}

const creditsAfter = await readAvailableCredits("after invocation");
console.log(
  JSON.stringify(
    {
      environment: "testnet",
      trust_mode: trustMode,
      tenant_did: tenantDid,
      contract_id: registered.contract_id,
      script_name: scriptName,
      registered_version: contractVersion,
      resolved_script_version: scriptVersion,
      wasm_bytes: wasm.byteLength,
      credits_before: creditsBefore,
      credits_after: creditsAfter,
      invocation: output,
      leak_check: "passed: four private fields absent from decoded response",
      sdk_findings: usageFindings,
    },
    null,
    2,
  ),
);
