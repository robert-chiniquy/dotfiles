# JSONL (Newline-Delimited JSON) Parsing

When working with large JSONL files (`.jsonl`, debug logs, event streams):

## Approach

1. **Never read the entire file** - Use line-by-line processing
2. **Use `head`/`tail` for sampling** - Get first N or last N records
3. **Use `wc -l` for counts** - Know how many records before processing
4. **Process with shell pipelines** - Combine tools for efficiency

## Common Patterns

### Count records
```bash
wc -l < file.jsonl
```

### Get first N records (preview)
```bash
head -n 5 file.jsonl | while read line; do echo "$line" | jq .; done
```

### Get last N records (recent)
```bash
tail -n 5 file.jsonl | while read line; do echo "$line" | jq .; done
```

### Extract specific fields from all records
```bash
cat file.jsonl | jq -r '.field1, .field2'
```

### Filter records by field value
```bash
cat file.jsonl | while read line; do
  if echo "$line" | jq -e '.status == "error"' > /dev/null 2>&1; then
    echo "$line" | jq .
  fi
done
```

### Summarize records (descriptions only)
```bash
cat file.jsonl | jq -r 'select(.description) | "#\(.entry_number // .id): \(.description)"'
```

### Process records N at a time (pagination)
```bash
# Records 10-20
sed -n '10,20p' file.jsonl | while read line; do echo "$line" | jq .; done
```

## For Agent Debug Logs

The agent notebook debug log has structure:
```json
{
  "entry_number": 1,
  "timestamp": "...",
  "session_id": "...",
  "description": "user's bug description",
  "raw_llm_response": "...",
  "streaming_text": "...",
  "error_message": "..."
}
```

### List all bug descriptions
```bash
cat "$TMPDIR/agent-notebook-debug.jsonl" | jq -r '"#\(.entry_number): \(.description)"'
```

### Get specific bug by entry number
```bash
sed -n '5p' "$TMPDIR/agent-notebook-debug.jsonl" | jq .
```

### Find bugs mentioning keyword
```bash
grep -i "workspace" "$TMPDIR/agent-notebook-debug.jsonl" | jq -r '"#\(.entry_number): \(.description)"'
```

## When Reading Large Records

If individual records are large (e.g., contain full LLM responses), use field selection:

```bash
# Get just the metadata, not content
cat file.jsonl | jq '{entry: .entry_number, desc: .description, error: .error_message}'
```

This avoids overwhelming context with large embedded content.
