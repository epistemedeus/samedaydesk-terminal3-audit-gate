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

    /// Host ABI gate for the 2026-09-17 Terminal 3 notice.
    ///
    /// `ap2/mandate` was removed from the testnet runtime. `host:tenant`
    /// `@1.2.0` is still the documented import for contracts that need tenant
    /// context, and this contract uses neither. A major `CONTRACT_VERSION`
    /// bump is therefore not applicable: it would change `policy_version`.
    #[test]
    fn world_imports_no_terminal3_host_interface() {
        let code = wit_code(include_str!("../wit/world.wit"));
        assert!(
            code.contains("package z:samedaydesk-audit@0.1.0;"),
            "package identity changed"
        );
        assert!(code.contains("plan-audit:"), "plan-audit export missing");
        for line in code.lines() {
            let trimmed = line.trim();
            assert!(
                !trimmed.starts_with("import "),
                "world.wit must not import a host interface: {trimmed}"
            );
        }
        for forbidden in [
            "ap2:",
            "ap2/",
            "host:tenant",
            "host:interfaces",
            "host:outbox",
            "host:session",
        ] {
            assert!(
                !code.contains(forbidden),
                "world.wit references removed or unused host package {forbidden}"
            );
        }
    }

    #[test]
    fn published_identity_stays_on_existing_slot() {
        assert_eq!(CONTRACT_VERSION, "0.1.0");
        let deploy = include_str!("../scripts/deploy.mjs");
        assert!(deploy.contains("const tail = \"sdd-audit\";"));
        assert!(deploy.contains("const contractVersion = \"0.1.0\";"));
        let package_json = include_str!("../package.json");
        assert!(package_json.contains("\"version\": \"0.1.0\""));
    }

    /// When a release component is already built, its custom sections must
    /// name `plan-audit` and must not name a Terminal 3 host package. Source
    /// control does not store the `.wasm`; `cargo test` before `cargo build`
    /// skips this half. `scripts/check-component-imports.sh` is the hard gate.
    #[test]
    fn built_component_has_no_terminal3_host_import() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/wasm32-wasip2/release/samedaydesk_audit_gate.wasm");
        if !path.is_file() {
            return;
        }
        let bytes = std::fs::read(&path).expect("read built component");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("z:samedaydesk-audit/contracts@0.1.0"),
            "export package version changed"
        );
        assert!(text.contains("plan-audit"));
        for forbidden in [
            "ap2:mandate",
            "ap2/mandate",
            "host:tenant",
            "host:interfaces",
            "host:outbox",
            "host:session",
        ] {
            assert!(
                !text.contains(forbidden),
                "built component contains {forbidden}"
            );
        }
    }

    fn wit_code(src: &str) -> String {
        let mut code = String::new();
        for line in src.lines() {
            let line = match line.find("//") {
                Some(index) => &line[..index],
                None => line,
            };
            code.push_str(line);
            code.push('\n');
        }
        code
    }
}
