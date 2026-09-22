#!/bin/sh
# Fail unless the built component exports plan-audit and imports only WASI.
# Terminal 3 host packages, including the removed ap2/mandate interface and
# host:tenant at any version, are rejected. Requires wasm-tools on PATH.
set -eu

wasm=${1:-target/wasm32-wasip2/release/samedaydesk_audit_gate.wasm}
if [ ! -f "$wasm" ]; then
  echo "component not found: $wasm" >&2
  exit 1
fi

wasm-tools validate --features component-model "$wasm"
wit=$(wasm-tools component wit "$wasm")

printf '%s\n' "$wit" | awk '
  $1 == "world" { in_world = 1; next }
  in_world && $0 ~ /^}/ { in_world = 0; next }
  in_world && $1 == "import" {
    imports++
    if ($2 !~ /^wasi:/) {
      printf "non-WASI import: %s\n", $0 > "/dev/stderr"
      bad = 1
    }
    if ($0 ~ /ap2:|ap2\/|host:tenant|host:interfaces|host:outbox|host:session/) {
      printf "forbidden host import: %s\n", $0 > "/dev/stderr"
      bad = 1
    }
    next
  }
  in_world && $1 == "export" {
    exports++
    if ($2 != "z:samedaydesk-audit/contracts@0.1.0;") {
      printf "unexpected export: %s\n", $0 > "/dev/stderr"
      bad = 1
    }
  }
  END {
    if (imports < 1 || exports != 1) {
      printf "expected WASI imports and one contracts export (imports=%d exports=%d)\n", imports + 0, exports + 0 > "/dev/stderr"
      bad = 1
    }
    if (bad) exit 1
  }
'

printf '%s\n' "$wit" | grep -q 'plan-audit: func'
echo "component imports accepted: WASI only, export z:samedaydesk-audit/contracts@0.1.0 plan-audit"
