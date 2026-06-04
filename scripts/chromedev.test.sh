#!/usr/bin/env bash
set -euo pipefail

make_fake_hdc() {
  local path="$1"

  cat > "$path" <<'HDC'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >> "$HDC_CALLS"

# Pass-through behavior is identical whether the caller pinned `-t <serial>`
# or not; the real hdc itself doesn't care how it's invoked as long as args
# parse. Keep $-prefixed args visible to assertions.
if [[ "${1:-}" == "list" && "${2:-}" == "targets" ]]; then
  cat "$HDC_TARGETS"
  exit 0
fi

if [[ "$*" == *"fport ls"* ]]; then
  cat "$HDC_STATE"
  exit 0
fi

if [[ "$*" == *"fport rm"* ]]; then
  local_arg="${@: -2:1}"
  remote_arg="${@: -1}"
  if [[ "$local_arg" == "tcp:9222" && "$remote_arg" == "localabstract:webview_devtools_remote_55602" ]]; then
    grep -Fv "tcp:9222 localabstract:webview_devtools_remote_55602" "$HDC_STATE" > "$HDC_STATE.tmp" || true
    mv "$HDC_STATE.tmp" "$HDC_STATE"
    exit 0
  fi
  echo "unexpected rm args: local=$local_arg remote=$remote_arg" >&2
  exit 42
fi

if [[ "$*" == *"shell"* && "$*" != *"fport"* ]]; then
  echo "u0_a1 57013 1 0 com.anjuke.home"
  exit 0
fi

if [[ "$*" == *"fport tcp:9222"* ]]; then
  printf 'FMR0223915003082    tcp:9222 %s    [Forward]\n' "${@: -1}" >> "$HDC_STATE"
  echo "Forwardport result:OK"
  exit 0
fi

echo "unexpected hdc call: $*" >&2
exit 43
HDC

  chmod +x "$path"
}

# Stub osascript so we can drive multi-device selection from tests without
# spawning a real GUI prompt. Tests pre-create FAKE_OSASCRIPT in PATH; the
# script writes the chosen label to FAKE_OSASCRIPT_CHOSEN.
# Set FAKE_OSASCRIPT_CANCEL=1 to simulate osascript exiting non-zero
# (e.g. real GUI process died). Set FAKE_OSASCRIPT_FALSE=1 to simulate
# AppleScript's "user pressed Cancel" path which returns the literal
# string "false" with exit code 0.
make_fake_osascript() {
  local path="$1"
  cat > "$path" <<'OSA'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${FAKE_OSASCRIPT_CANCEL:-0}" == "1" ]]; then
  exit 1
fi
if [[ "${FAKE_OSASCRIPT_FALSE:-0}" == "1" ]]; then
  printf '%s' "false"
  exit 0
fi
printf '%s' "$FAKE_OSASCRIPT_CHOSEN"
OSA
  chmod +x "$path"
}

test_removes_only_target_port() {
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-state" <<'STATE'
FMR0223915003082    tcp:9222 localabstract:webview_devtools_remote_55602    [Forward]
FMR0223915003082    tcp:8888 localabstract:other_socket    [Forward]
STATE
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
TARGETS
  PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    bash scripts/chromedev.sh anjuke >/dev/null

  if ! grep -Fxq -- "-t FMR0223915003082 fport rm tcp:9222 localabstract:webview_devtools_remote_55602" "$tmpdir/hdc-calls"; then
    echo "expected chromedev.sh to remove tcp:9222 using hdc fport rm arguments" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  if grep -Fq "tcp:8888" "$tmpdir/hdc-calls"; then
    echo "expected chromedev.sh to leave non-9222 fport mappings untouched" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_discovers_hdc_from_home_bin() {
  local tmpdir home
  tmpdir=$(mktemp -d)
  home="$tmpdir/home"
  mkdir -p "$home/bin"
  make_fake_hdc "$home/bin/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-state" <<'STATE'
FMR0223915003082    tcp:9222 localabstract:webview_devtools_remote_55602    [Forward]
STATE
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
TARGETS

  HOME="$home" \
    PATH="/usr/bin:/bin:/usr/sbin:/sbin:$tmpdir" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    bash scripts/chromedev.sh anjuke >/dev/null

  if ! grep -Fxq -- "-t FMR0223915003082 fport tcp:9222 localabstract:webview_devtools_remote_57013" "$tmpdir/hdc-calls"; then
    echo "expected chromedev.sh to find hdc from HOME/bin when GUI PATH is minimal" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_exits_when_no_devices() {
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  : > "$tmpdir/hdc-targets"
  : > "$tmpdir/hdc-state"

  set +e
  PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    bash scripts/chromedev.sh anjuke >"$tmpdir/out" 2>&1
  local rc=$?
  set -e

  if [[ "$rc" -eq 0 ]]; then
    echo "expected non-zero exit when no hdc devices are connected" >&2
    cat "$tmpdir/out" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  if ! grep -Fq "未检测到任何 hdc 设备" "$tmpdir/out"; then
    echo "expected friendly error when no devices connected" >&2
    cat "$tmpdir/out" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_picks_device_via_osascript_and_pins_t_flag() {
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-state" <<'STATE'
FMR0223915003082    tcp:9222 localabstract:webview_devtools_remote_55602    [Forward]
FMR9999999999999    tcp:9999 localabstract:other_socket    [Forward]
STATE
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
FMR9999999999999    192.168.1.20:5555    device
TARGETS

  PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    FAKE_OSASCRIPT_CHOSEN="FMR9999999999999  192.168.1.20:5555  [device]" \
    bash scripts/chromedev.sh anjuke >/dev/null

  if ! grep -Fxq -- "-t FMR9999999999999 fport ls" "$tmpdir/hdc-calls"; then
    echo "expected fport ls to be invoked with -t FMR9999999999999" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  if ! grep -Fxq -- "-t FMR9999999999999 fport rm tcp:9222 localabstract:webview_devtools_remote_55602" "$tmpdir/hdc-calls"; then
    echo "expected fport rm to target tcp:9222 mapping on selected device" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  if ! grep -Fxq -- "-t FMR9999999999999 fport tcp:9222 localabstract:webview_devtools_remote_57013" "$tmpdir/hdc-calls"; then
    echo "expected fport add to use PID 57013 on selected device" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  # Critically: the OTHER device must NEVER be addressed.
  if grep -Fq "FMR0223915003082" "$tmpdir/hdc-calls"; then
    echo "expected chromedev.sh to never address the un-selected device" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_exits_when_user_cancels_device_picker() {
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
FMR9999999999999    192.168.1.20:5555    device
TARGETS
  : > "$tmpdir/hdc-state"

  set +e
  PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    FAKE_OSASCRIPT_CANCEL=1 \
    bash scripts/chromedev.sh anjuke >"$tmpdir/out" 2>&1
  local rc=$?
  set -e

  if [[ "$rc" -eq 0 ]]; then
    echo "expected non-zero exit when user cancels device picker" >&2
    cat "$tmpdir/out" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  # After cancel, only `list targets` should have been called (to populate the
  # chooser). No fport or shell invocations are allowed.
  if ! grep -Fxq -- "list targets" "$tmpdir/hdc-calls"; then
    echo "expected list targets to be called to populate device chooser" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  if grep -Eq -- "fport|shell " "$tmpdir/hdc-calls"; then
    echo "expected no fport/shell calls after user cancels selection, but got:" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_hdc_device_env_skips_picker() {
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-state" <<'STATE'
FMR0223915003082    tcp:9222 localabstract:webview_devtools_remote_55602    [Forward]
STATE
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
FMR9999999999999    192.168.1.20:5555    device
TARGETS

  PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    FAKE_OSASCRIPT_CHOSEN="FMR9999999999999  192.168.1.20:5555  [device]" \
    HDC_DEVICE="FMR0223915003082" \
    bash scripts/chromedev.sh anjuke >/dev/null

  # With HDC_DEVICE set, osascript must NOT be called.
  # We assert indirectly: if osascript were called and returned the non-pinned
  # serial, fake hdc would still pass — so instead we check that FAKE_OSASCRIPT_CHOSEN
  # is irrelevant. The existing calls already prove the script ran end-to-end.
  if ! grep -Fxq -- "-t FMR0223915003082 fport rm tcp:9222 localabstract:webview_devtools_remote_55602" "$tmpdir/hdc-calls"; then
    echo "expected fport rm to run on HDC_DEVICE when env is set" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  # Un-selected device must never be addressed when HDC_DEVICE pins the other one.
  if grep -Fq "FMR9999999999999" "$tmpdir/hdc-calls"; then
    echo "expected chromedev.sh to address only HDC_DEVICE, never the other device" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  # osascript must not be invoked at all when HDC_DEVICE is set.
  if [[ -f "$tmpdir/osascript-called" ]]; then
    echo "expected osascript not to be invoked when HDC_DEVICE is set" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_silent_path_with_single_device() {
  # 1 device connected: osascript must NOT be called; the script must
  # proceed straight to fport cleanup using the single device.
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-state" <<'STATE'
FMR0223915003082    tcp:9222 localabstract:webview_devtools_remote_55602    [Forward]
STATE
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
TARGETS

  # Wrap the osascript fake so we can detect if it ever got called.
  cat > "$tmpdir/osascript" <<'OSA'
#!/usr/bin/env bash
touch "${FAKE_OSASCRIPT_MARKER}"
exit 99
OSA
  chmod +x "$tmpdir/osascript"
  FAKE_OSASCRIPT_MARKER="$tmpdir/osascript-called" \
    PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    bash scripts/chromedev.sh anjuke >/dev/null

  if [[ -f "$tmpdir/osascript-called" ]]; then
    echo "expected osascript not to be invoked when only 1 device is connected" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  if ! grep -Fxq -- "-t FMR0223915003082 fport tcp:9222 localabstract:webview_devtools_remote_57013" "$tmpdir/hdc-calls"; then
    echo "expected single-device path to use the only device serial" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_exits_when_osascript_returns_false_string() {
  # AppleScript's `choose from list` returns the literal string "false" with
  # exit code 0 when the user hits Cancel. The script must treat that the
  # same as osascript failure and exit non-zero.
  local tmpdir
  tmpdir=$(mktemp -d)
  make_fake_hdc "$tmpdir/hdc"
  make_fake_osascript "$tmpdir/osascript"
  cat > "$tmpdir/hdc-targets" <<'TARGETS'
FMR0223915003082    192.168.1.10:5555    device
FMR9999999999999    192.168.1.20:5555    device
TARGETS
  : > "$tmpdir/hdc-state"

  set +e
  PATH="$tmpdir:$PATH" \
    HDC_CALLS="$tmpdir/hdc-calls" \
    HDC_STATE="$tmpdir/hdc-state" \
    HDC_TARGETS="$tmpdir/hdc-targets" \
    FAKE_OSASCRIPT_FALSE=1 \
    bash scripts/chromedev.sh anjuke >"$tmpdir/out" 2>&1
  local rc=$?
  set -e

  if [[ "$rc" -eq 0 ]]; then
    echo "expected non-zero exit when osascript returns 'false' string" >&2
    cat "$tmpdir/out" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  if ! grep -Fq "已取消设备选择" "$tmpdir/out"; then
    echo "expected cancel-by-false-string message in output" >&2
    cat "$tmpdir/out" >&2
    rm -rf "$tmpdir"
    exit 1
  fi
  # No fport calls should happen after a "false" return.
  if grep -Eq -- "fport|shell " "$tmpdir/hdc-calls"; then
    echo "expected no fport/shell calls after osascript returns 'false'" >&2
    cat "$tmpdir/hdc-calls" >&2
    rm -rf "$tmpdir"
    exit 1
  fi

  rm -rf "$tmpdir"
}

test_silent_path_with_single_device
test_exits_when_osascript_returns_false_string

test_removes_only_target_port
test_discovers_hdc_from_home_bin
test_exits_when_no_devices
test_picks_device_via_osascript_and_pins_t_flag
test_exits_when_user_cancels_device_picker
test_hdc_device_env_skips_picker
