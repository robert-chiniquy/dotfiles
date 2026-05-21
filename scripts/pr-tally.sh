#!/usr/bin/env bash
# pr-tally.sh -- markdown PR tally for the authenticated user. Sections:
#   CREATED              PRs you opened today (or on the given date)
#   MERGED               PRs you authored that merged today (or on the given date)
#   MERGEABLE            Currently-open PRs strictly ready to merge:
#                        mergeStateStatus==CLEAN, at least one reviewer's
#                        latest review is APPROVED, and nobody's latest
#                        review is CHANGES_REQUESTED.
#   APPROVED, NOT CLEAN  Currently-open PRs approved by a reviewer with no
#                        merge conflicts, but mergeStateStatus != CLEAN.
#                        Typically BLOCKED (branch protection wants thread
#                        resolution) or UNSTABLE (a non-required check
#                        failed). State column shows which.
#
# Both review-based sections are date-independent. Open-PR scan limit 500.
#
# Why latestReviews instead of reviewDecision: reviewDecision is null
# when the repo has no branch protection rule requiring reviews -- even
# if a human has actually approved. We want "a human approved it" as
# the gate, regardless of branch protection.
#
# Usage:
#   pr-tally.sh                  # today
#   pr-tally.sh 2026-05-15       # specific date for CREATED/MERGED
#   pr-tally.sh --json [DATE]
#
# Requires: gh CLI, jq.

set -euo pipefail

DATE="${1:-}"
JSON=0
if [[ "${DATE:-}" == "--json" ]]; then
  JSON=1
  DATE="${2:-}"
fi
if [[ -z "${DATE}" ]]; then
  DATE="$(date -u +%Y-%m-%d)"
fi

ME="$(gh api user --jq .login)"
fields='number,title,repository,closedAt,createdAt,state,url'

created_open=$(gh search prs   --author "${ME}" --created   "${DATE}" --state open   --limit 100 --json "${fields}")
created_closed=$(gh search prs --author "${ME}" --created   "${DATE}" --state closed --limit 100 --json "${fields}")
merged=$(gh search prs         --author "${ME}" --merged-at "${DATE}" --state closed --limit 100 --json "${fields}")
all_open=$(gh search prs       --author "${ME}" --state open --limit 500 --json "${fields}")

GRAPHQL_QUERY='
query($owner: String!, $repo: String!, $num: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $num) {
      mergeable
      mergeStateStatus
      latestReviews(first: 50) { nodes { state } }
    }
  }
}'

# Echoes "clean<TAB>STATE" if the PR is strictly mergeable (CLEAN + approved),
# "approved<TAB>STATE" if approved with no conflicts but mergeStateStatus !=
# CLEAN (BLOCKED, UNSTABLE, BEHIND, ...), or empty otherwise.
classify_pr() {
  local owner="$1" repo="$2" num="$3"
  local resp
  resp=$(gh api graphql \
    -f query="${GRAPHQL_QUERY}" \
    -F owner="${owner}" -F repo="${repo}" -F num="${num}" 2>/dev/null) || return 0

  jq -r '
    .data.repository.pullRequest as $pr
    | ($pr.latestReviews.nodes // []) as $lr
    | if ($pr.mergeable == "MERGEABLE")
        and ($lr | any(.state == "APPROVED"))
        and ($lr | all(.state != "CHANGES_REQUESTED"))
      then
        (if $pr.mergeStateStatus == "CLEAN" then "clean" else "approved" end)
        + "\t" + ($pr.mergeStateStatus // "")
      else ""
      end
  ' <<<"${resp}"
}

mergeable_json='[]'
approved_json='[]'
while IFS= read -r pr; do
  owner_repo=$(jq -r '.repository.nameWithOwner' <<<"${pr}")
  owner="${owner_repo%/*}"
  repo="${owner_repo#*/}"
  num=$(jq -r '.number' <<<"${pr}")
  IFS=$'\t' read -r bucket state <<<"$(classify_pr "${owner}" "${repo}" "${num}")"
  case "${bucket}" in
    clean)
      mergeable_json=$(jq --argjson p "${pr}" '. + [$p]' <<<"${mergeable_json}")
      ;;
    approved)
      augmented=$(jq --arg s "${state}" '. + {mergeStateStatus: $s}' <<<"${pr}")
      approved_json=$(jq --argjson p "${augmented}" '. + [$p]' <<<"${approved_json}")
      ;;
  esac
done < <(jq -c '.[]' <<<"${all_open}")

if [[ "${JSON}" == 1 ]]; then
  jq -n \
    --argjson created_open "${created_open}" \
    --argjson created_closed "${created_closed}" \
    --argjson merged "${merged}" \
    --argjson mergeable "${mergeable_json}" \
    --argjson approved_not_clean "${approved_json}" \
    --arg date "${DATE}" \
    --arg me "${ME}" \
    '{
      date: $date,
      me: $me,
      created: ($created_open + $created_closed),
      merged: $merged,
      mergeable: $mergeable,
      approved_not_clean: $approved_not_clean
    }'
  exit 0
fi

count() { jq 'length' <<<"$1"; }

c_total=$(( $(count "${created_open}") + $(count "${created_closed}") ))
m_total=$(count "${merged}")
r_total=$(count "${mergeable_json}")
a_total=$(count "${approved_json}")

printf '# Tally — %s — @%s\n\n' "${DATE}" "${ME}"
printf -- '- **CREATED**: %d\n'             "${c_total}"
printf -- '- **MERGED**: %d\n'              "${m_total}"
printf -- '- **MERGEABLE**: %d\n'           "${r_total}"
printf -- '- **APPROVED, NOT CLEAN**: %d\n\n' "${a_total}"

print_section() {
  local heading="$1"
  local json="$2"
  local include_state="${3:-0}"
  local n
  n=$(count "${json}")
  if (( n == 0 )); then return 0; fi
  printf '## %s\n\n' "${heading}"

  local jq_row headers
  if (( include_state )); then
    jq_row='.[] | [.url, (.repository.nameWithOwner | esc), (.number | tostring), (.mergeStateStatus // ""), (.title | esc)] | @tsv'
    headers='URL Repo # State Title'
  else
    jq_row='.[] | [.url, (.repository.nameWithOwner | esc), (.number | tostring), (.title | esc)] | @tsv'
    headers='URL Repo # Title'
  fi

  # Emit TSV, then have awk compute per-column widths and emit a padded
  # markdown table so the raw source aligns in a terminal.
  jq -r "def esc: gsub(\"\\\\|\"; \"\\\\|\"); ${jq_row}" <<<"${json}" \
    | awk -F'\t' -v hdrs="${headers}" '
      BEGIN { ncol = split(hdrs, h, " "); for (i=1;i<=ncol;i++) w[i]=length(h[i]) }
      {
        for (i=1;i<=ncol;i++) {
          r[NR,i] = $i
          if (length($i) > w[i]) w[i] = length($i)
        }
        n = NR
      }
      END {
        printf "|"; for (i=1;i<=ncol;i++) printf " %-*s |", w[i], h[i]; printf "\n"
        printf "|"
        for (i=1;i<=ncol;i++) {
          s = ""
          for (j=0;j<w[i]+2;j++) s = s "-"
          printf "%s|", s
        }
        printf "\n"
        for (k=1;k<=n;k++) {
          printf "|"
          for (i=1;i<=ncol;i++) printf " %-*s |", w[i], r[k,i]
          printf "\n"
        }
      }
    '
  printf '\n'
}

print_section "CREATED"             "$(jq -s 'add' <<<"${created_open}${created_closed}")"
print_section "MERGED"              "${merged}"
print_section "MERGEABLE"           "${mergeable_json}"
print_section "APPROVED, NOT CLEAN" "${approved_json}" 1
