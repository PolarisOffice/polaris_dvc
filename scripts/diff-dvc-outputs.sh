#!/usr/bin/env bash
# Soft-diff upstream DVC.exe outputs against polaris' committed
# expected.json files. Used by .github/workflows/dvc-parity.yml.
#
# Usage:
#   scripts/diff-dvc-outputs.sh <golden_root> <dvc_output_dir>
#
# <golden_root>     : directory containing `<case>/expected.json`
# <dvc_output_dir>  : directory containing `<case>.json` produced by DVC.exe
#
# Writes a human-readable summary to stdout. Exits 0 regardless of
# divergences — the workflow uses this as diagnostic output, not a
# gating check.

set -u

golden_root=${1:?golden_root argument required}
dvc_dir=${2:?dvc_output_dir argument required}

pass=0
fail=0
missing=0
cases=()

for case_dir in "$golden_root"/*/; do
    name=$(basename "$case_dir")
    [[ "$name" = "_dvc-output" ]] && continue
    [[ ! -f "$case_dir/expected.json" ]] && continue
    cases+=("$name")
done

for name in "${cases[@]}"; do
    expected="$golden_root/$name/expected.json"
    actual="$dvc_dir/$name.json"

    if [[ ! -f "$actual" ]]; then
        printf '%-50s %s\n' "$name" "MISSING (DVC.exe produced no output file)"
        missing=$((missing + 1))
        continue
    fi

    if diff -q "$expected" "$actual" > /dev/null 2>&1; then
        printf '%-50s %s\n' "$name" "MATCH"
        pass=$((pass + 1))
    else
        printf '%-50s %s\n' "$name" "DIFFER"
        fail=$((fail + 1))
    fi
done

echo
echo "--- Summary ---"
echo "Match    : $pass"
echo "Differ   : $fail"
echo "Missing  : $missing"
echo "Total    : ${#cases[@]}"
echo

if (( fail > 0 )); then
    echo "--- Per-case diffs (first 30 lines each) ---"
    for name in "${cases[@]}"; do
        actual="$dvc_dir/$name.json"
        expected="$golden_root/$name/expected.json"
        [[ ! -f "$actual" ]] && continue
        if ! diff -q "$expected" "$actual" > /dev/null 2>&1; then
            echo
            echo "### $name"
            diff -u "$expected" "$actual" | head -n 30 || true
        fi
    done
fi

# Always succeed — this is a diagnostic tool.
exit 0
