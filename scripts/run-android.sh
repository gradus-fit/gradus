#!/usr/bin/env bash
# Start an Android emulator and run the Gradus Flutter app with hot reload.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
client_dir="$repo_root/client"
app_dir="$client_dir/app"

usage() {
  cat <<'EOF'
Usage: scripts/run-android.sh [emulator-id]

Uses an already-running Android emulator when one is available. Otherwise,
starts the supplied emulator ID, or the sole configured Android emulator.
EOF
}

if (( $# > 1 )); then
  usage >&2
  exit 2
fi

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if ! command -v flutter >/dev/null; then
  echo "Flutter is not available on PATH." >&2
  exit 1
fi

if ! command -v jq >/dev/null; then
  echo "jq is required to identify the running Android emulator." >&2
  exit 1
fi

if ! command -v adb >/dev/null; then
  echo "adb is required to wait for the Android emulator to finish booting." >&2
  exit 1
fi

if [[ ! -d "$app_dir/android" ]]; then
  echo "Android platform files are missing. Run this once:" >&2
  echo "  cd \"$app_dir\" && flutter create --platforms=android ." >&2
  exit 1
fi

running_emulator() {
  flutter devices --machine |
    jq -r '.[] | select(.emulator == true and (.targetPlatform | startswith("android-"))) | .id' |
    head -n 1
}

configured_emulators() {
  flutter emulators |
    awk -F '•' '/•/ { id = $1; platform = $4; gsub(/^[[:space:]]+|[[:space:]]+$/, "", id); gsub(/^[[:space:]]+|[[:space:]]+$/, "", platform); if (id != "Id" && platform == "android") print id }'
}

wait_for_device() {
  for _ in {1..60}; do
    device_id="$(running_emulator)"
    if [[ -n "$device_id" ]]; then
      return 0
    fi
    sleep 2
  done
  return 1
}

wait_for_boot() {
  for _ in {1..60}; do
    if [[ "$(adb -s "$device_id" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == "1" ]]; then
      return 0
    fi
    sleep 2
  done
  return 1
}

if ! emulator_output="$(configured_emulators)"; then
  echo "Unable to list configured Android emulators." >&2
  exit 1
fi

emulators=()
while IFS= read -r emulator; do
  [[ -n "$emulator" ]] && emulators+=("$emulator")
done <<< "$emulator_output"

requested_emulator="${1:-}"
if [[ -n "$requested_emulator" ]]; then
  emulator_found=false
  for emulator in "${emulators[@]}"; do
    if [[ "$emulator" == "$requested_emulator" ]]; then
      emulator_found=true
      break
    fi
  done
  if [[ "$emulator_found" != true ]]; then
    echo "Unknown Android emulator: $requested_emulator" >&2
    printf 'Available Android emulators:\n  %s\n' "${emulators[@]}" >&2
    exit 1
  fi
fi

device_id="$(running_emulator)"
if [[ -z "$device_id" ]]; then
  if [[ -z "$requested_emulator" ]]; then
    if (( ${#emulators[@]} == 0 )); then
      echo "No Android emulators are configured. Create one in Android Studio's Device Manager." >&2
      exit 1
    fi

    if (( ${#emulators[@]} > 1 )); then
      echo "Multiple Android emulators are configured. Choose one:" >&2
      printf '  %s\n' "${emulators[@]}" >&2
      echo "Run: $0 <emulator-id>" >&2
      exit 1
    fi

    requested_emulator="${emulators[0]}"
  fi

  echo "Starting Android emulator: $requested_emulator"
  flutter emulators --launch "$requested_emulator"

  echo "Waiting for the emulator to be detected..."
  if ! wait_for_device; then
    echo "The emulator did not become available within two minutes." >&2
    exit 1
  fi
fi

echo "Waiting for Android to finish booting..."
if ! wait_for_boot; then
  echo "The emulator did not finish booting within two minutes." >&2
  exit 1
fi

echo "Running Gradus on Android device: $device_id"
(
  cd "$client_dir"
  flutter pub get
)

cd "$app_dir"
exec flutter run -d "$device_id"
