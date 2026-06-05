#!/usr/bin/env bash
# pr-tally.sh -- markdown PR tally for the authenticated user. Sections:
#   CREATED                PRs you opened today (or on the given date)
#   MERGED                 PRs you authored that merged today (or on date)
#   MERGEABLE              Currently-open PRs strictly ready to merge:
#                          mergeStateStatus==CLEAN, at least one reviewer's
#                          latest review is APPROVED, and nobody's latest
#                          review is CHANGES_REQUESTED.
#   APPROVED, NOT CLEAN    Currently-open PRs approved by a reviewer with no
#                          merge conflicts, but mergeStateStatus != CLEAN.
#                          Typically BLOCKED (branch protection wants thread
#                          resolution) or UNSTABLE (a non-required check
#                          failed). State column shows which.
#   READY FOR REVIEW       Currently-open PRs awaiting a first review:
#                          not draft, CI rollup state == SUCCESS (or no
#                          checks configured), no reviewer's latest is
#                          APPROVED, no reviewer's latest is CHANGES_REQUESTED.
#
# All three review-based sections are date-independent and mutually
# exclusive. Open-PR scan limit 500.
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
      isDraft
      mergeable
      mergeStateStatus
      latestReviews(first: 50) { nodes { state author { login } } }
      commits(last: 1) {
        nodes { commit { statusCheckRollup { state } } }
      }
    }
  }
}'

# Echoes a single-line JSON object:
#   {"bucket": "clean" | "approved" | "ready" | "",
#    "state":  "<mergeStateStatus or empty>",
#    "approvers": ["login1", "login2", ...]}
# Bucket meaning:
#   clean    -- strictly mergeable (mergeStateStatus==CLEAN + approved)
#   approved -- approved with no conflicts but mergeStateStatus != CLEAN
#   ready    -- awaiting first review, CI green, not draft
#   ""       -- none of the above
# Buckets are mutually exclusive. "approvers" is only populated for the
# clean/approved buckets and lists the login of every reviewer whose
# latest review state is APPROVED.
classify_pr() {
  local owner="$1" repo="$2" num="$3"
  local resp
  resp=$(gh api graphql \
    -f query="${GRAPHQL_QUERY}" \
    -F owner="${owner}" -F repo="${repo}" -F num="${num}" 2>/dev/null) || return 0

  jq -c '
    .data.repository.pullRequest as $pr
    | ($pr.latestReviews.nodes // []) as $lr
    | ($lr | map(select(.state == "APPROVED") | .author.login)) as $approvers
    | ($lr | any(.state == "APPROVED"))         as $approved
    | ($lr | any(.state == "CHANGES_REQUESTED")) as $changes_req
    | ($pr.commits.nodes[0].commit.statusCheckRollup.state // "SUCCESS") as $ci
    | if $approved and ($pr.mergeable == "MERGEABLE") and ($changes_req | not) then
        { bucket: (if $pr.mergeStateStatus == "CLEAN" then "clean" else "approved" end),
          state: ($pr.mergeStateStatus // ""),
          approvers: $approvers }
      elif ($approved | not) and ($changes_req | not) and ($pr.isDraft == false) and ($ci == "SUCCESS") then
        { bucket: "ready", state: ($pr.mergeStateStatus // ""), approvers: [] }
      else
        { bucket: "", state: "", approvers: [] }
      end
  ' <<<"${resp}"
}

mergeable_json='[]'
approved_json='[]'
ready_json='[]'
while IFS= read -r pr; do
  owner_repo=$(jq -r '.repository.nameWithOwner' <<<"${pr}")
  owner="${owner_repo%/*}"
  repo="${owner_repo#*/}"
  num=$(jq -r '.number' <<<"${pr}")
  cls=$(classify_pr "${owner}" "${repo}" "${num}")
  bucket=$(jq -r '.bucket' <<<"${cls}")
  case "${bucket}" in
    clean)
      augmented=$(jq --argjson c "${cls}" '. + {approvers: $c.approvers}' <<<"${pr}")
      mergeable_json=$(jq --argjson p "${augmented}" '. + [$p]' <<<"${mergeable_json}")
      ;;
    approved)
      augmented=$(jq --argjson c "${cls}" '. + {mergeStateStatus: $c.state, approvers: $c.approvers}' <<<"${pr}")
      approved_json=$(jq --argjson p "${augmented}" '. + [$p]' <<<"${approved_json}")
      ;;
    ready)
      ready_json=$(jq --argjson p "${pr}" '. + [$p]' <<<"${ready_json}")
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
    --argjson ready_for_review "${ready_json}" \
    --arg date "${DATE}" \
    --arg me "${ME}" \
    '{
      date: $date,
      me: $me,
      created: ($created_open + $created_closed),
      merged: $merged,
      mergeable: $mergeable,
      approved_not_clean: $approved_not_clean,
      ready_for_review: $ready_for_review
    }'
  exit 0
fi

count() { jq 'length' <<<"$1"; }

c_total=$(( $(count "${created_open}") + $(count "${created_closed}") ))
m_total=$(count "${merged}")
r_total=$(count "${mergeable_json}")
a_total=$(count "${approved_json}")
rfr_total=$(count "${ready_json}")

printf '# Tally — %s — @%s\n\n' "${DATE}" "${ME}"
printf -- '- **CREATED**: %d\n'                  "${c_total}"
printf -- '- **MERGED**: %d\n'                   "${m_total}"
printf -- '- **MERGEABLE**: %d\n'                "${r_total}"
printf -- '- **APPROVED, NOT CLEAN**: %d\n'      "${a_total}"
printf -- '- **READY FOR REVIEW, CI GREEN**: %d\n\n' "${rfr_total}"

print_section() {
  local heading="$1"
  local json="$2"
  local show_approvers="${3:-0}"
  local n
  n=$(count "${json}")
  if (( n == 0 )); then return 0; fi
  printf '## %s\n\n' "${heading}"
  if (( show_approvers )); then
    jq -r '
      .[] | (
        "- " + .url,
        ((.approvers // []) | map("  - " + .) | .[])
      )
    ' <<<"${json}"
  else
    jq -r '.[] | "- " + .url' <<<"${json}"
  fi
  printf '\n'
}

print_section "CREATED"                    "$(jq -s 'add' <<<"${created_open}${created_closed}")"
print_section "MERGED"                     "${merged}"
print_section "MERGEABLE"                  "${mergeable_json}" 1
print_section "APPROVED, NOT CLEAN"        "${approved_json}"
print_section "READY FOR REVIEW, CI GREEN" "${ready_json}"
