---
name: jsonl-parsing
description: |
  Patterns for working with large JSONL files, debug logs, and event streams.
  Use when processing .jsonl files, agent debug logs, or newline-delimited
  JSON data. Covers line-by-line processing, sampling, filtering, and
  field extraction.
---

# JSONL (Newline-Delimited JSON) Parsing

When working with large JSONL files (.jsonl, debug logs, event streams):

1. Never read the entire file -- use line-by-line processing
2. Use head/tail for sampling
3. Use wc -l for counts
4. Process with shell pipelines

Common patterns: count records, preview first/last N, extract fields with jq, filter by field value, paginate with sed.
