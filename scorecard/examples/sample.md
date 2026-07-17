# Release readiness
sub: Cut candidate for the 0.9 train — where each gate stands.
meta: freeze Fri   ·   ship bar all-green, no P0 open   ·   weights must x3 · should x2
score: 148/175
pass: 123
note: on track — the two at-risk gates are both in review
footer: Directional snapshot. Refresh from the tracker before the go/no-go.

## Must Have (x3)
| id | state | score | criterion | note |
|----|-------|-------|-----------|------|
| R1 | solid | 5 | Data migration is reversible | Down-migrations tested on a prod clone; rollback under 5 min. |
| R2 | risk  | 3 | p99 latency within budget | 210ms vs 200ms under peak; caching [PR-482](https://example.com/pr/482) in review. |
| R3 | solid | 5 | No plaintext secrets in logs | Scrubber on all sinks; verified on the staging firehose. |
| R4 | gap   | 0 | Rollout is feature-flagged | Flag exists but the kill switch is untested in prod. |

## Should Have (x2)
| id | state | score | criterion | note |
|----|-------|-------|-----------|------|
| Q1 | solid | 5 | Dashboards cover the new path | Golden-signal panels added; alerts wired. |
| Q2 | risk  | 3 | Runbook updated | Draft exists; on-call has not reviewed it. |

## Callouts
| STANDOUT | R4 kill switch is the only red — untested in prod, blocks the go decision. |
| NEXT | Land the caching PR (R2), dry-run the kill switch (R4), on-call reads the runbook (Q2). |
