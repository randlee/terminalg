#!/usr/bin/env bash
set -euo pipefail

REG=".claude/agents/registry.yaml"

frontmatter() {
  awk 'f{print} /^---/{f=1;c++} c==2{exit}' "$1" | sed '1d'
}

if ! command -v yq >/dev/null 2>&1; then
  echo "ERROR: yq is required for validation. Install via 'brew install yq'." >&2
  exit 1
fi

fail=0

for agent in .claude/agents/*.md; do
  name=$(basename "$agent" .md)
  file_ver=$(frontmatter "$agent" | yq -r '.version' || echo "")
  reg_ver=$(yq -r ".agents.\"$name\".version" "$REG" || echo "")
  reg_path=$(yq -r ".agents.\"$name\".path" "$REG" || echo "")

  if [[ -z "$file_ver" || -z "$reg_ver" ]]; then
    echo "ERROR: $name missing version (file='$file_ver' registry='$reg_ver')" >&2
    fail=1
  fi

  if [[ -n "$reg_path" && "$reg_path" != ".claude/agents/$name.md" ]]; then
    echo "ERROR: $name path mismatch: registry=$reg_path expected=.claude/agents/$name.md" >&2
    fail=1
  fi

  if [[ -n "$file_ver" && -n "$reg_ver" && "$file_ver" != "$reg_ver" ]]; then
    echo "ERROR: $name version mismatch: file=$file_ver registry=$reg_ver" >&2
    fail=1
  fi

done

if [[ $fail -eq 0 ]]; then
  echo "All agent versions validated"
fi

exit $fail
