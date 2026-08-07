//! SameDayDesk Audit Gate for the Terminal 3 T3N sandbox.
//!
//! The contract turns a private AI search audit intake into a deterministic,
//! public-safe action plan. It declares no Terminal 3 custom host interfaces,
//! network egress, or durable storage. Sensitive intake fields are parsed
//! inside the TEE contract and are never copied into the response.
#![warn(clippy::style, missing_debug_implementations)]
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

extern crate alloc;

pub const CONTRACT_VERSION: &str = "0.1.0";

wit_bindgen::generate!({
    world: "samedaydesk-audit",
    path: "wit",
    additional_derives: [
        serde::Deserialize,
        serde::Serialize,
    ],
    generate_all,
});

mod audit;

struct Component;

#[cfg(target_arch = "wasm32")]
impl exports::z::samedaydesk_audit::contracts::Guest for Component {
    fn plan_audit(
        req: exports::z::samedaydesk_audit::contracts::GenericInput,
    ) -> Result<alloc::vec::Vec<u8>, alloc::string::String> {
        let input = req.input.ok_or("plan-audit: missing input")?;
        audit::plan_audit(&input)
    }
}

#[cfg(target_arch = "wasm32")]
export!(Component);

#[cfg(test)]
mod tests {
    use super::CONTRACT_VERSION;

    #[test]
    fn contract_version_is_semver() {
        let parts: Vec<&str> = CONTRACT_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "CONTRACT_VERSION must be MAJOR.MINOR.PATCH");
        for part in parts {
            assert!(part.parse::<u32>().is_ok(), "each part must be a number");
        }
    }

    #[test]
    fn contract_version_matches_package() {
        assert_eq!(CONTRACT_VERSION, env!("CARGO_PKG_VERSION"));
    }
}
