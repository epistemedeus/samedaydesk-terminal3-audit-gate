//! Deterministic audit planning with explicit output minimization.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

const MAX_PRIVATE_TEXT_BYTES: usize = 4_096;
const MAX_DOMAIN_BYTES: usize = 253;
const MAX_EMAIL_BYTES: usize = 320;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditIntake {
    public_id: String,
    business_domain: String,
    contact_email: String,
    internal_notes: String,
    monthly_budget_cents: u64,
    signals: ReadinessSignals,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadinessSignals {
    has_organization_schema: bool,
    has_local_business_schema: bool,
    has_person_authority_schema: bool,
    has_faq_schema: bool,
    has_llms_txt: bool,
    has_indexable_service_pages: bool,
    authority_evidence_count: u8,
    location_consistency_pct: u8,
}

#[derive(Debug, Serialize)]
struct AuditPlan {
    public_id: String,
    readiness_score: u8,
    readiness_tier: &'static str,
    priority_actions: Vec<&'static str>,
    private_fields_withheld: u8,
    policy_version: &'static str,
}

pub fn plan_audit(input: &[u8]) -> Result<Vec<u8>, String> {
    let intake: AuditIntake =
        serde_json::from_slice(input).map_err(|e| alloc::format!("plan-audit: bad input: {e}"))?;
    validate(&intake)?;

    let score = score(&intake.signals);
    let plan = AuditPlan {
        public_id: intake.public_id,
        readiness_score: score,
        readiness_tier: tier(score),
        priority_actions: actions(&intake.signals),
        private_fields_withheld: 4,
        policy_version: crate::CONTRACT_VERSION,
    };

    serde_json::to_vec(&plan).map_err(|e| alloc::format!("plan-audit: encode output: {e}"))
}

fn validate(intake: &AuditIntake) -> Result<(), String> {
    if intake.public_id.is_empty() || intake.public_id.len() > 64 {
        return Err("plan-audit: public_id must contain 1 to 64 characters".into());
    }
    if !intake
        .public_id
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(
            "plan-audit: public_id may use only letters, numbers, dot, dash, and underscore".into(),
        );
    }
    if intake.business_domain.is_empty() || intake.business_domain.len() > MAX_DOMAIN_BYTES {
        return Err("plan-audit: business_domain length is invalid".into());
    }
    if intake.contact_email.is_empty() || intake.contact_email.len() > MAX_EMAIL_BYTES {
        return Err("plan-audit: contact_email length is invalid".into());
    }
    if intake.internal_notes.len() > MAX_PRIVATE_TEXT_BYTES {
        return Err("plan-audit: internal_notes exceeds 4096 bytes".into());
    }
    if intake.monthly_budget_cents > 100_000_000 {
        return Err("plan-audit: monthly_budget_cents exceeds policy limit".into());
    }
    if intake.signals.location_consistency_pct > 100 {
        return Err("plan-audit: location_consistency_pct must be 0 to 100".into());
    }
    Ok(())
}

fn score(s: &ReadinessSignals) -> u8 {
    let mut value = 0u8;
    value += if s.has_organization_schema { 15 } else { 0 };
    value += if s.has_local_business_schema { 15 } else { 0 };
    value += if s.has_person_authority_schema { 10 } else { 0 };
    value += if s.has_faq_schema { 10 } else { 0 };
    value += if s.has_llms_txt { 5 } else { 0 };
    value += if s.has_indexable_service_pages { 15 } else { 0 };
    value += s.authority_evidence_count.min(5) * 2;
    value += s.location_consistency_pct / 5;
    value.min(100)
}

fn tier(score: u8) -> &'static str {
    match score {
        80..=100 => "ready",
        60..=79 => "promising",
        35..=59 => "weak",
        _ => "critical",
    }
}

fn actions(s: &ReadinessSignals) -> Vec<&'static str> {
    let mut out = Vec::new();
    if !s.has_organization_schema {
        out.push("Publish Organization JSON-LD with a stable legal and brand identity");
    }
    if !s.has_local_business_schema {
        out.push("Connect each real location to LocalBusiness structured data");
    }
    if !s.has_person_authority_schema {
        out.push("Connect named experts, credentials, and reviewed services in structured data");
    }
    if !s.has_indexable_service_pages {
        out.push("Create crawlable service pages that answer high-intent customer questions");
    }
    if s.location_consistency_pct < 85 {
        out.push("Reconcile location names, addresses, and phone data across owned profiles");
    }
    if s.authority_evidence_count < 3 {
        out.push("Add verifiable third-party authority evidence and cite its original source");
    }
    if !s.has_faq_schema {
        out.push("Add concise answer-shaped FAQs with matching visible page content");
    }
    if !s.has_llms_txt {
        out.push("Publish a small llms.txt index after the primary entity gaps are fixed");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(signals: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "public_id": "lead-001",
            "business_domain": "private-prospect.example",
            "contact_email": "owner@private-prospect.example",
            "internal_notes": "Confidential expansion goal and decision-maker context",
            "monthly_budget_cents": 75000,
            "signals": signals,
        }))
        .unwrap()
    }

    fn complete_signals() -> serde_json::Value {
        serde_json::json!({
            "has_organization_schema": true,
            "has_local_business_schema": true,
            "has_person_authority_schema": true,
            "has_faq_schema": true,
            "has_llms_txt": true,
            "has_indexable_service_pages": true,
            "authority_evidence_count": 5,
            "location_consistency_pct": 100,
        })
    }

    #[test]
    fn complete_intake_scores_one_hundred() {
        let output = plan_audit(&sample(complete_signals())).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["readiness_score"], 100);
        assert_eq!(value["readiness_tier"], "ready");
        assert_eq!(value["priority_actions"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn weak_intake_returns_ordered_actions() {
        let output = plan_audit(&sample(serde_json::json!({
            "has_organization_schema": false,
            "has_local_business_schema": false,
            "has_person_authority_schema": false,
            "has_faq_schema": false,
            "has_llms_txt": false,
            "has_indexable_service_pages": false,
            "authority_evidence_count": 0,
            "location_consistency_pct": 25,
        })))
        .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["readiness_score"], 5);
        assert_eq!(value["readiness_tier"], "critical");
        assert_eq!(value["priority_actions"].as_array().unwrap().len(), 8);
    }

    #[test]
    fn output_withholds_every_private_field() {
        let input = sample(complete_signals());
        let output = plan_audit(&input).unwrap();
        let rendered = String::from_utf8(output).unwrap();
        for forbidden in [
            "private-prospect.example",
            "owner@private-prospect.example",
            "Confidential expansion goal",
            "75000",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "leaked private field: {forbidden}"
            );
        }
        assert!(rendered.contains("lead-001"));
        assert!(rendered.contains("\"private_fields_withheld\":4"));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&sample(complete_signals())).unwrap();
        value["unexpected"] = serde_json::json!("not admitted");
        let err = plan_audit(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(err.contains("unknown field"));
    }

    #[test]
    fn public_id_is_constrained() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&sample(complete_signals())).unwrap();
        value["public_id"] = serde_json::json!("lead 001<script>");
        let err = plan_audit(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(err.contains("letters, numbers"));
    }

    #[test]
    fn impossible_percentage_is_rejected() {
        let mut signals = complete_signals();
        signals["location_consistency_pct"] = serde_json::json!(101);
        let err = plan_audit(&sample(signals)).unwrap_err();
        assert!(err.contains("0 to 100"));
    }
}
