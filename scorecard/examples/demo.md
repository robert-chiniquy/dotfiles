# Scorecard feature tour
sub: Built-in rendering demo — the selected scorecard follows below.
meta: all mode · typed Markdown model · OSC 8 links · image/text chart fallback
score: 18/25
pass: 18
note: every visual state and chart type in one command
footer: Demo data only. No source file is mutated and no tracker lookup runs for this card.
groups: launch=30, reliability=20, polish=10, callouts=5, footer=-1

## Launch gates (x3)
| id | state | score | criterion | note |
| --- | --- | ---: | --- | --- |
| CORE | solid | 5 | Structural document model | Rows, links, groups, and charts are typed before rendering. | grp:launch | grp:reliability |
| [PR42](https://github.com/example/scorecard/pull/42) | risk | 3 | Linked review gate | OSC 8 link plus aligned columns and a deliberately longer justification. | grp:launch |
| LIN7 | gap | 0 | External decision | Waiting on [DEMO-7](https://linear.app/example/issue/DEMO-7/choose-the-release-window). | grp:launch |

## Supporting signals (x1)
| id | state | score | criterion | note |
| --- | --- | ---: | --- | --- |
| OBS | solid | 10 | Wide score column | Multi-group membership keeps related rows together during fit. | grp:reliability | grp:polish |
| COPY | risk | 1 | Compact narrative | Missing chart values stay missing instead of becoming zero. | grp:polish |

## Chart: Completion trend
type: sparkline
| day | complete | confidence |
| --- | ---: | ---: |
| Mon | 20% | 35% |
| Tue | 38% | n/a |
| Wed | 55% | 61% |
| Thu | 72% | 78% |
| Fri | 88% | 91% |

## Chart: Latency distribution
type: histogram
| milliseconds |
| ---: |
| 12 |
| 14 |
| 16 |
| 16 |
| 19 |
| 23 |
| 27 |
| 31 |
| 44 |

## Chart: Throughput over time
type: time-series
| hour | accepted | rejected |
| --- | ---: | ---: |
| 09:00 | 120 | 18 |
| 10:00 | 180 | 14 |
| 11:00 | 240 | 9 |
| 12:00 | 310 | 6 |
| 13:00 | 390 | 4 |

## Callouts
| STANDOUT | One source drives the boxed card, text charts, and inline PNG charts. | grp:polish |
| BLOCK | GAP rows remain visually unmistakable without changing the underlying data. | grp:launch |
| NEXT | Run [scorecard demo](https://github.com/example/scorecard) in iTerm2 and through a pipe to compare both renderers. |
