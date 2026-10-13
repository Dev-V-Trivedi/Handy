#!/usr/bin/env bash
# CI wrapper for `tauri`: runs it verbosely and, on failure, republishes the
# tail of the output as ::error:: annotations. Annotations are visible on the
# run page without signing in, unlike raw job logs.
set -uo pipefail
log="$(mktemp)"
bun tauri "$@" --verbose 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  # Lines that look like the real cause first, then the raw tail.
  grep -n -i -E "error|failed|couldn't|could not|no such|denied|codesign|bundle_dmg|hdiutil" "$log" | tail -12 |
    while IFS= read -r line; do echo "::error title=tauri cause::${line:0:400}"; done
  tail -n 25 "$log" | sed 's/\r$//' | awk 'BEGIN{ORS="%0A"} {print substr($0,1,300)}' > "$log.tail"
  echo "::error title=tauri output tail::$(cat "$log.tail")"
fi
exit "$status"
