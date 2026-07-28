#!/usr/bin/env sh
# 6LAYER security gate: the hardening checklist of SECURITY.md section 13.
set -eu
export CI=true
export GIT_TERMINAL_PROMPT=0
export GIT_PAGER=cat
export PAGER=cat
export DEBIAN_FRONTEND=noninteractive
export CARGO_TERM_COLOR=never
export CARGO_INCREMENTAL=0
export RUST_BACKTRACE=1
export MC_HEADLESS=1
fail() { echo "security check: FAIL - $1" >&2; exit 1; }

# 1. forbid(unsafe_code) in the deterministic crates.
for c in mc_core mc_data; do
  f="crates/$c/src/lib.rs"
  [ -f "$f" ] || continue
  grep -q 'forbid(unsafe_code)' "$f" || fail "missing #![forbid(unsafe_code)] in $f"
done

# 2. mc_core purity (INV-01, INV-02, INV-03). Absence beats prohibition.
if [ -d crates/mc_core/src ]; then
  if grep -rnE '\bf32\b|\bf64\b|HashMap|HashSet|SystemTime|std::thread|thread_rng' crates/mc_core/src >/dev/null 2>&1; then
    grep -rnE '\bf32\b|\bf64\b|HashMap|HashSet|SystemTime|std::thread|thread_rng' crates/mc_core/src >&2
    fail "mc_core contains a determinism-breaking construct (listed above)"
  fi
fi

# 3. No networking crate anywhere in the tree (INV-09).
if command -v cargo >/dev/null 2>&1 && [ -f Cargo.lock ]; then
  for n in reqwest hyper tokio-tungstenite ureq curl socket2 async-std-net; do
    if grep -q "^name = \"$n\"$" Cargo.lock; then fail "networking crate present in Cargo.lock: $n"; fi
  done
fi

# 4. Single point of file access (INV-07).
if [ -d crates ]; then
  hits=$(grep -rn 'File::open\|File::create\|fs::read\|fs::write' crates --include=*.rs 2>/dev/null \
         | grep -v 'mc_shell/src/fsroot.rs' | grep -v 'mc_tools/src/' | grep -v '/tests/' | grep -v '/benches/' || true)
  if [ -n "$hits" ]; then printf '%s\n' "$hits" >&2; fail "file access outside fsroot::confine (listed above)"; fi
fi

# 5. Committed-secret scan. There are no secrets in this project; keep it that way.
pat='(BEGIN [A-Z ]*PRIVATE KEY|aws_secret_access_key|AKIA[0-9A-Z]{16}|xox[abpr]-|sk-[A-Za-z0-9]{20,}|ghp_[A-Za-z0-9]{30,})'
if git ls-files 2>/dev/null | grep -v '^\.env\.example$' | xargs grep -lEI "$pat" 2>/dev/null | grep . >&2; then
  fail "possible committed secret (files listed above)"
fi
if git ls-files 2>/dev/null | grep -qx '.env'; then fail ".env is tracked by git"; fi

# 6. Dependency policy.
if command -v cargo-deny >/dev/null 2>&1 || cargo deny --version >/dev/null 2>&1; then
  cargo deny check advisories bans licenses sources
fi

# 7. Fuzz corpora present and non-empty once EP-006 has landed.
if [ -d fuzz/fuzz_targets ]; then
  for t in fuzz_save fuzz_content fuzz_tape; do
    [ -f "fuzz/fuzz_targets/$t.rs" ] || fail "missing fuzz target: $t"
    n=$(ls "fuzz/corpus/$t" 2>/dev/null | wc -l)
    [ "$n" -gt 0 ] || fail "empty fuzz corpus: $t"
  done
fi

# 8. The two security tests exist once EP-006 has landed.
if [ -d crates/mc_shell/tests ]; then
  for t in no_socket log_redaction fsroot_confine; do
    [ -f "crates/mc_shell/tests/$t.rs" ] || fail "missing security test: $t.rs"
  done
fi

# 9. mc_core must never gain a logging dependency (it would bring a clock).
if [ -f crates/mc_core/Cargo.toml ]; then
  grep -q '^tracing' crates/mc_core/Cargo.toml && fail "mc_core has a logging dependency"
fi
echo "security check: ok"
