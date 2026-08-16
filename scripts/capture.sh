#!/usr/bin/env bash
# The packet capture that is Phase 4's gate evidence — R-25, S-10, and part of S-07b.
#
#   scripts/capture.sh selftest               # the report script, checked in both directions
#   scripts/capture.sh probe                  # what the service does: padding, and the known hit
#   scripts/capture.sh off  [--minutes 10]    # gate line 4: zero packets with the setting off
#   scripts/capture.sh on                     # gate lines 1, 2 and 6: the check on the wire
#
# Each run writes a directory under `captures/` holding the pcap, the sockets the app's process
# tree held while it ran, what the machine was at the time, and `report.md` — which is the thing
# that gets summarized into the session log of `trustvault-state.md`.
#
# ── Why this exists as a script ────────────────────────────────────────────────────────────────
#
# The gate asks for a capture, and a capture taken by hand is taken once, by one person, with the
# strings they remembered to grep for. Four of the seven gate lines rest on it, and nothing in CI
# can run the application at all — so the only defence against "we looked and it seemed fine" is
# that the looking is written down and repeatable. What is scripted here is the whole procedure:
# the fixture vault, the forbidden-string list generated from it, the capture, the attribution,
# and the verdict. The person supplies the clicks.
#
# ── Attributing packets to the app: what this does and what it costs (D-88) ────────────────────
#
# `tcpdump` has no notion of a PID, so "tcpdump on the app's PID" has to be built. This captures
# **everything** on every interface with no filter, and separately samples the sockets held by the
# app's **process tree** every 200 ms; the report attributes a packet to the app when either
# endpoint matches a sampled socket.
#
# The process tree, not the process, and that is the part worth reading twice. A Tauri app on
# Linux is WebKitGTK, and WebKitGTK does its networking in a separate `WebKitNetworkProcess`. A
# capture attributed to the main PID alone would have watched the one process in the application
# least likely to open a socket, and missed the one most likely to.
#
# What it costs: sampling can miss a socket that opens and closes inside 200 ms. That is why the
# capture is unfiltered and why `capture-report.py` runs the forbidden-string sweep and the
# hostname search over **every** byte it recorded rather than over the attributed subset. A missed
# sample can understate what the app did; it cannot hide a password.
#
# The alternatives are in the decision log. Briefly: a network namespace would make attribution
# exact by construction, and it needs the GUI to reach the display across a netns — X11's abstract
# socket is namespaced, `slirp4netns` is not installed here, and a root veth with NAT changes the
# routing the capture is supposed to be measuring. nftables `socket cgroupv2` matching is exact
# for output too, and it is a root firewall rule plus nflog plumbing for a run that happens twice.
#
# ── What must be true before running `off` or `on` ─────────────────────────────────────────────
#
# * A release build of the app: `npm run tauri build -- --no-bundle`. Not the `measure` build —
#   D-65's inspector is a pane no user has and a process that talks to nothing.
# * The fixture vault and its manifest, which this script builds if they are missing:
#   `cargo run --release --example audit-vault -p trustvault-core --features benchfixture`.
#   Twenty-one items, twelve distinct values, master password `correct horse battery staple`.
# * `sudo`, for tcpdump alone. **The app itself is never run as root** — it would write into
#   root's config directory and unlock a vault as the wrong user.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
repository="$PWD"

binary="target/release/trustvault"
vault="${TRUSTVAULT_CAPTURE_VAULT:-/tmp/trustvault-audit.tvault}"
manifest="${TRUSTVAULT_CAPTURE_MANIFEST:-/tmp/trustvault-audit-manifest.txt}"
minutes=10
prefix=5BAA6
# The published SHA-1 of `password`, whose prefix is the one the fixture guarantees is in the run.
known_suffix=1E4C9B93F3F0682250B6CF8331B7EE68FD8
sample_interval=0.2

mode="${1:-}"
shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --minutes) minutes="$2"; shift 2 ;;
    --binary) binary="$2"; shift 2 ;;
    --vault) vault="$2"; shift 2 ;;
    --manifest) manifest="$2"; shift 2 ;;
    --prefix) prefix="$2"; shift 2 ;;
    -h|--help) sed -n '2,12p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

say() { printf '\n\033[1m%s\033[0m\n' "$*"; }
note() { printf '  %s\n' "$*"; }

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "missing: $1 — $2" >&2; exit 2; }
}

# ── probe: the service, not the app ────────────────────────────────────────────────────────────
#
# Gate line 3 is a claim about what HIBP does — a padded response carries zero-count rows and is
# larger than the unpadded one for the same prefix — and it cannot be read off a capture of the
# app at all, because both responses are opaque inside TLS. It is measured here with curl, which
# is honest about what it is: two requests this project made deliberately, from outside the
# application, to hold the service to something.
probe() {
  need curl "the probe asks the live service two questions"
  say "Probing $prefix against api.pwnedpasswords.com"
  local directory
  directory="$repository/captures/probe-$(date +%Y%m%d-%H%M%S)"
  mkdir -p "$directory"

  curl -sS -D "$directory/padded.headers" -H 'Add-Padding: true' \
    "https://api.pwnedpasswords.com/range/$prefix" -o "$directory/padded.txt"
  curl -sS -D "$directory/unpadded.headers" \
    "https://api.pwnedpasswords.com/range/$prefix" -o "$directory/unpadded.txt"

  local padded_rows unpadded_rows padded_zero unpadded_zero padded_bytes unpadded_bytes hit
  padded_rows=$(grep -c ':' "$directory/padded.txt" || true)
  unpadded_rows=$(grep -c ':' "$directory/unpadded.txt" || true)
  padded_zero=$(grep -c ':0$' <(tr -d '\r' < "$directory/padded.txt") || true)
  unpadded_zero=$(grep -c ':0$' <(tr -d '\r' < "$directory/unpadded.txt") || true)
  padded_bytes=$(wc -c < "$directory/padded.txt")
  unpadded_bytes=$(wc -c < "$directory/unpadded.txt")
  hit=$(tr -d '\r' < "$directory/padded.txt" | awk -F: -v s="$known_suffix" '$1==s {print $2}')

  {
    echo "# HIBP service probe — $prefix, $(date -Iseconds)"
    echo
    echo "| | Padded | Unpadded |"
    echo "|---|---|---|"
    echo "| Rows | $padded_rows | $unpadded_rows |"
    echo "| Zero-count rows (padding) | $padded_zero | $unpadded_zero |"
    echo "| Bytes | $padded_bytes | $unpadded_bytes |"
    echo
    echo "- \`vary\` header on the padded response: $(grep -i '^vary:' "$directory/padded.headers" | tr -d '\r' || echo 'absent')"
    echo "- \`cache-control\`: $(grep -i '^cache-control:' "$directory/padded.headers" | tr -d '\r' || echo 'absent')"
    echo "- Count for \`$prefix$known_suffix\` (the SHA-1 of \`password\`): ${hit:-not present}"
    echo
    if [[ $padded_zero -gt 0 && $padded_bytes -gt $unpadded_bytes ]]; then
      echo "**Gate line 3: pass** — the padded response carries $padded_zero zero-count rows and is"
      echo "$((padded_bytes - unpadded_bytes)) bytes larger than the unpadded response for the same prefix."
    else
      echo "**Gate line 3: FAIL** — padding did not produce zero-count rows, or did not grow the response."
    fi
    echo
    echo "Row counts are recorded, never asserted as a band: the decoy count for one prefix moved"
    echo "110 → 125 → 156 → 163 across two days of measuring (D-77)."
  } | tee "$directory/report.md"

  say "Wrote $directory/report.md"
}

# ── the fixture vault ──────────────────────────────────────────────────────────────────────────
build_fixture() {
  if [[ -f "$vault" && -f "$manifest" ]]; then
    note "fixture vault: $vault (already there)"
    return
  fi
  if [[ -f "$vault" || -f "$manifest" ]]; then
    echo "one of $vault / $manifest exists without the other — delete both and re-run" >&2
    exit 2
  fi
  say "Building the fixture vault"
  cargo run --release --quiet --example audit-vault -p trustvault-core --features benchfixture \
    -- "$vault" "$manifest"
}

# ── the capture runs ───────────────────────────────────────────────────────────────────────────
capture() {
  local setting="$1"
  need tcpdump "the capture itself"
  need python3 "the report"
  need ss "socket attribution"
  need sudo "tcpdump needs root; the app does not and must not have it"

  [[ -x "$binary" ]] || { echo "no build at $binary — run: npm run tauri build -- --no-bundle" >&2; exit 2; }
  build_fixture

  local directory
  directory="$repository/captures/$setting-$(date +%Y%m%d-%H%M%S)"
  mkdir -p "$directory"

  say "Breach checking must be turned $setting in the app before the run"
  if [[ "$setting" == off ]]; then
    note "Settings → Watchtower → 'Check passwords against Have I Been Pwned' unchecked (the default)."
    note "Then use the app normally for $minutes minutes: unlock, browse, search, copy, edit, scan."
  else
    note "Settings → Watchtower → the same row checked."
    note "Then: unlock $vault, open Watchtower, and run the breach check to completion."
  fi
  printf '\nPress Enter when you have read the above. '
  read -r _

  sudo -v

  {
    echo "mode:      $setting"
    echo "date:      $(date -Iseconds)"
    echo "host:      $(uname -srmo)"
    echo "binary:    $binary"
    echo "build:     $(git -C "$repository" rev-parse --short HEAD)$(git -C "$repository" diff --quiet || echo ' + uncommitted changes')"
    echo "vault:     $vault"
    echo "manifest:  $manifest"
    echo "resolver:  $(grep -m1 '^nameserver' /etc/resolv.conf 2>/dev/null || echo unknown)"
    echo "hibp A:    $(getent ahostsv4 api.pwnedpasswords.com 2>/dev/null | awk '{print $1}' | sort -u | paste -sd, || echo 'not resolved')"
    echo "proxy env: ${ALL_PROXY:-unset} / ${HTTPS_PROXY:-unset} / ${https_proxy:-unset}"
    echo "interval:  ${sample_interval}s socket sampling"
  } > "$directory/run.txt"
  cat "$directory/run.txt"

  # tcpdump under its own pid file, because `sudo kill` has to reach tcpdump rather than sudo.
  # No BPF filter, on purpose: the report's negative claims are only worth something if nothing
  # was excluded before they were made.
  say "Starting tcpdump on every interface"
  sudo sh -c "echo \$\$ > '$directory/tcpdump.pid'; exec tcpdump -i any -s 0 -U -n -w '$directory/capture.pcap' -Z '$USER'" \
    > "$directory/tcpdump.log" 2>&1 &

  for _ in $(seq 1 50); do
    grep -q 'listening on' "$directory/tcpdump.log" 2>/dev/null && break
    sleep 0.1
  done
  grep -q 'listening on' "$directory/tcpdump.log" || { echo "tcpdump did not start — see $directory/tcpdump.log" >&2; exit 2; }
  note "$(tr -d '\n' < "$directory/tcpdump.log")"

  stop_capture() {
    local pid
    pid=$(cat "$directory/tcpdump.pid" 2>/dev/null || true)
    [[ -n "$pid" ]] && sudo kill -INT "$pid" 2>/dev/null || true
    sleep 0.5
  }
  trap 'stop_capture' EXIT

  say "Launching $binary"
  "$binary" &
  local app=$!
  note "pid $app — quit the app from its own window when you are done"

  # Sample the sockets of the whole process tree. Re-derived every pass rather than once: the
  # network process is spawned after the window opens, and a tree computed at launch would not
  # contain the process most likely to make a request.
  (
    while kill -0 "$app" 2>/dev/null; do
      tree=$(
        ps -eo pid=,ppid= | awk -v root="$app" '
          { parent[$1] = $2 }
          END {
            for (pid in parent) {
              p = pid
              for (hops = 0; hops < 64 && p != "" && p != "1"; hops++) {
                if (p == root) { print pid; break }
                p = parent[p]
              }
            }
            print root
          }'
      )
      pattern=$(echo "$tree" | sort -u | sed 's/^/pid=/' | paste -sd'|')
      if [[ -n "$pattern" ]]; then
        ss -tunapH 2>/dev/null | grep -E "$pattern" | while IFS= read -r line; do
          printf '%s\t%s\n' "$(date +%s.%N)" "$line"
        done >> "$directory/sockets.txt"
      fi
      sleep "$sample_interval"
    done
  ) &
  local sampler=$!

  if [[ "$setting" == off ]]; then
    say "Recording for $minutes minutes — use the app"
    local seconds=$((minutes * 60))
    local elapsed=0
    while [[ $elapsed -lt $seconds ]] && kill -0 "$app" 2>/dev/null; do
      sleep 5
      elapsed=$((elapsed + 5))
      printf '\r  %d:%02d of %d:00 ' $((elapsed / 60)) $((elapsed % 60)) "$minutes"
    done
    printf '\n'
    if kill -0 "$app" 2>/dev/null; then
      note "time is up — quit the app from its window"
    else
      note "the app exited before the $minutes minutes were up; the run is short and the report says so"
    fi
  else
    say "Run the breach check, then quit the app"
  fi

  wait "$app" 2>/dev/null || true
  kill "$sampler" 2>/dev/null || true
  wait "$sampler" 2>/dev/null || true
  stop_capture
  trap - EXIT

  touch "$directory/sockets.txt"
  say "Analysing"
  set +e
  python3 scripts/capture-report.py \
    --mode "$setting" \
    --pcap "$directory/capture.pcap" \
    --sockets "$directory/sockets.txt" \
    --manifest "$manifest" \
    --meta "$directory/run.txt" \
    --out "$directory/report.md"
  local status=$?
  set -e

  say "Wrote $directory/report.md"
  note "Paste its verdict table into the session log of trustvault-state.md — the gate asks for"
  note "the evidence recorded, not asserted."
  return $status
}

case "$mode" in
  # The gate's evidence is this script's output, so the script has evidence of its own. It builds
  # a capture by hand, analyses it, and then breaks it on purpose: a planted password, a full
  # SHA-1 in hex and in raw binary, a ClientHello naming somewhere else. Needs no root, no build
  # and no network, which is what makes it runnable on every machine rather than on this one.
  selftest) need python3 "the report"; python3 scripts/capture-report.py --self-test ;;
  probe) probe ;;
  off) capture off ;;
  on) capture on ;;
  *)
    sed -n '2,12p' "${BASH_SOURCE[0]}"
    exit 2
    ;;
esac
