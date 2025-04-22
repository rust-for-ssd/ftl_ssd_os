#!/usr/bin/env bash
#
# validate_pipe_names.sh – ensure the same pipe names appear in both files
#
# Usage: ./validate_pipe_names.sh <file1> <file2>
#
# Exits with:
#   0  if the sets of pipe names are identical
#   1  if incorrect usage
#   2  if any line in file1 or file2 is blank or malformed (optional)
#   3  if there are pipe names in one file but not the other

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <file1> <file2>"
  exit 1
fi

file1="$1"
file2="$2"

# Extract pipe names (first field before the space), skip empty lines
mapfile -t names1 < <(awk 'NF>0 {print $1}' "$file1" | sort -u)
mapfile -t names2 < <(awk 'NF>0 {print $1}' "$file2" | sort -u)

# Use comm to find differences
missing_in_2=$(comm -23 <(printf "%s\n" "${names1[@]}") \
                    <(printf "%s\n" "${names2[@]}"))
missing_in_1=$(comm -13 <(printf "%s\n" "${names1[@]}") \
                    <(printf "%s\n" "${names2[@]}"))

if [[ -n "$missing_in_2" ]] || [[ -n "$missing_in_1" ]]; then
  echo "Pipe‑name consistency check FAILED:"
  if [[ -n "$missing_in_2" ]]; then
    echo "  In $file1 but missing in $file2:"
    echo "$missing_in_2" | sed 's/^/    • /'
  fi
  if [[ -n "$missing_in_1" ]]; then
    echo "  In $file2 but missing in $file1:"
    echo "$missing_in_1" | sed 's/^/    • /'
  fi
  exit 3
else
  echo "All pipe names are present in both files."
  exit 0
fi

