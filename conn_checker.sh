#!/usr/bin/env bash
#
# validate.sh – check that each non-empty line in the given file
# matches the pattern: name name,name
#
# Usage: ./validate.sh <file>
#

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <file>"
  exit 1
fi

file="$1"

# Define the regex: 
#   ^            start of line
#   [[:alnum:]_]+  one or more letters, digits or underscores
#   \  exactly one space
#   [[:alnum:]_]+  one or more letters/digits/underscores
#   ,            exactly one comma
#   [[:alnum:]_]+  one or more letters/digits/underscores
#   $            end of line
re='^[[:alnum:]_]+ [[:alnum:]_]+,[[:alnum:]_]+$'

ok=true
line_no=0

while IFS= read -r line || [[ -n "$line" ]]; do
  ((line_no++))
  # Skip empty lines
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
