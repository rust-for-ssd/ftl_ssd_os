#!/usr/bin/env bash
#
# validate_stages.sh – check that each non‑empty line in the given file
# matches the pattern: pipe_name stage1[,stage2,...]
#
# Usage: ./validate_stages.sh <file>
#

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <file>"
  exit 1
fi

file="$1"

# Regex breakdown:
#   ^                start of line
#   [[:alnum:]_]+      pipe_name: 1+ letters/digits/underscores
#   ␣                exactly one space
#   [[:alnum:]_]+      first stage name
#   (,[[:alnum:]_]+)*  zero or more of “,stage_name”
#   $                end of line
re='^[[:alnum:]_]+ [[:alnum:]_]+(,[[:alnum:]_]+)*$'

ok=true
line_no=0

while IFS= read -r line || [[ -n "$line" ]]; do
  ((line_no++))
  # skip empty lines
  [[ -z "$line" ]] && continue

  if [[ ! $line =~ $re ]]; then
    echo "Line $line_no invalid: '$line'"
    ok=false
  fi
done < "$file"

if $ok; then
  echo "All lines are correctly formatted."
  exit 0
else
  echo "Formatting errors found."
  exit 2
fi

