#!/usr/bin/env python3
"""Aggregate locally recorded agent token usage by Git repository.

The default seven-day report reads local stores only. Direct Codex, Claude
Code, and Grok logs have dedicated adapters; Pi and OpenCode are
provider-agnostic, so xAI, Gemini, Claude, OpenAI, and local-model usage is
counted whenever the client persisted token metadata. Stores without both
token and repository fields remain visible in diagnostics instead of being
reported as zero.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple
from urllib.parse import unquote


SOURCE_ORDER = (
    "claude_code",
    "codex",
    "grok",
    "pi",
    "opencode",
    "crush",
    "squire",
    "cursor",
)
SOURCE_NAMES = {
    "claude_code": "Claude Code",
    "codex": "Codex",
    "grok": "Grok",
    "pi": "Pi",
    "opencode": "OpenCode",
    "crush": "Crush",
    "squire": "Squire",
    "cursor": "Cursor",
}


@dataclass(frozen=True)
class Window:
    start: datetime
    end: datetime

    def __post_init__(self) -> None:
        if self.start.tzinfo is None or self.end.tzinfo is None:
            raise ValueError("window timestamps must include a timezone")
        if self.end < self.start:
            raise ValueError("window end precedes start")

    def contains(self, value: datetime) -> bool:
        return self.start <= value <= self.end


@dataclass(frozen=True)
class StorePaths:
    repo_home: Path
    codex: Path
    claude: Path
    grok: Path
    pi: Path
    opencode: Path
    crush: Path
    cursor: Path
    squire_metrics: Path


@dataclass(frozen=True)
class UsageRecord:
    source: str
    tokens: int
    timestamp: datetime
    repo: Optional[str] = None
    provider: Optional[str] = None
    model: Optional[str] = None
    session: Optional[str] = None
    event_id: Optional[str] = None


@dataclass
class AdapterOutput:
    records: List[UsageRecord]
    diagnostics: Dict[str, Any]


def default_store_paths(home: Path, repo_home: Optional[Path] = None) -> StorePaths:
    repo_root = repo_home or home / "repo"
    return StorePaths(
        repo_home=repo_root,
        codex=home / ".codex" / "sessions",
        claude=home / ".claude" / "projects",
        grok=home / ".grok",
        pi=home / ".pi" / "agent" / "sessions",
        opencode=home / ".local" / "share" / "opencode" / "storage",
        crush=home / ".config" / "crush" / "agent-sessions-v2",
        cursor=home / ".cursor" / "projects",
        squire_metrics=repo_root / "dotfiles" / "scripts" / "squire-metrics.jsonl",
    )


def parse_time(value: Any) -> Optional[datetime]:
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        seconds = float(value) / 1000.0 if abs(float(value)) >= 100_000_000_000 else float(value)
        try:
            return datetime.fromtimestamp(seconds, tz=timezone.utc)
        except (OverflowError, OSError, ValueError):
            return None
    if not isinstance(value, str) or not value:
        return None
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed


def integer(value: Any) -> Optional[int]:
    if isinstance(value, bool):
        return None
    if isinstance(value, int):
        return value
    if isinstance(value, float) and value.is_integer():
        return int(value)
    return None


def remote_name(remote: Any) -> Optional[str]:
    if not isinstance(remote, str) or not remote.strip():
        return None
    value = remote.strip().rstrip("/")
    name = value.rsplit("/", 1)[-1]
    if ":" in name:
        name = name.rsplit(":", 1)[-1]
    if name.endswith(".git"):
        name = name[:-4]
    return name or None


class RepoResolver:
    def __init__(self, repo_home: Path) -> None:
        self.repo_home = repo_home
        try:
            entries = [entry.name for entry in repo_home.iterdir() if entry.is_dir()]
        except OSError:
            entries = []
        self.known_repos = sorted(entries, key=len, reverse=True)
        self.cache: Dict[Tuple[str, str, str], Optional[str]] = {}

    def resolve(
        self,
        cwd: Any = None,
        source_path: Optional[Path] = None,
        remote: Any = None,
    ) -> Optional[str]:
        cwd_text = cwd if isinstance(cwd, str) else ""
        source_text = str(source_path) if source_path else ""
        remote_text = remote if isinstance(remote, str) else ""
        key = (cwd_text, source_text, remote_text)
        if key in self.cache:
            return self.cache[key]

        name = remote_name(remote)
        if name:
            self.cache[key] = name
            return name

        if cwd_text:
            candidate = Path(cwd_text)
            try:
                relative = candidate.resolve().relative_to(self.repo_home.resolve())
                if relative.parts:
                    self.cache[key] = relative.parts[0]
                    return relative.parts[0]
            except (OSError, ValueError):
                pass

            if candidate.exists():
                try:
                    completed = subprocess.run(
                        ["git", "-C", cwd_text, "config", "--get", "remote.origin.url"],
                        check=False,
                        capture_output=True,
                        text=True,
                        timeout=2,
                    )
                    name = remote_name(completed.stdout) if completed.returncode == 0 else None
                    if name:
                        self.cache[key] = name
                        return name
                except (OSError, subprocess.TimeoutExpired):
                    pass

        for candidate_text in (cwd_text, source_text):
            name = self._from_text(candidate_text)
            if name:
                self.cache[key] = name
                return name

        self.cache[key] = None
        return None

    def _from_text(self, value: str) -> Optional[str]:
        if not value:
            return None
        if value in self.known_repos:
            return value
        for marker in ("Users-rch-repo-", "repo-", "src-"):
            start = 0
            while True:
                position = value.find(marker, start)
                if position < 0:
                    break
                tail = value[position + len(marker) :]
                for repo in self.known_repos:
                    if tail == repo or tail.startswith(repo + "-") or tail.startswith(repo + "/"):
                        return repo
                start = position + len(marker)
        for match in re.finditer(r"/(?:repo|src)/([^/]+)", value):
            name = match.group(1)
            if not self.known_repos or name in self.known_repos:
                return name
        return None


def provider_from_model(model: Any) -> str:
    value = model.lower() if isinstance(model, str) else ""
    if "grok" in value:
        return "xai"
    if "claude" in value:
        return "anthropic"
    if "gemini" in value:
        return "google"
    if re.search(r"(^|[/_-])(gpt|codex|o[134])([/_-]|$)", value):
        return "openai"
    if any(name in value for name in ("qwen", "llama", "mistral", "ollama")):
        return "local"
    return "unknown"


def normalize_provider(provider: Any, model: Any = None) -> Optional[str]:
    value = provider.strip().lower() if isinstance(provider, str) else ""
    aliases = {
        "x-ai": "xai",
        "x.ai": "xai",
        "google-ai": "google",
        "google_genai": "google",
        "ollama": "local",
    }
    value = aliases.get(value, value)
    if value:
        return value
    inferred = provider_from_model(model)
    return None if inferred == "unknown" else inferred


def recent_files(root: Path, pattern: str, start: datetime, recursive: bool = True) -> List[Path]:
    if not root.exists():
        return []
    iterator = root.rglob(pattern) if recursive else root.glob(pattern)
    cutoff = start.timestamp()
    result: List[Path] = []
    for path in iterator:
        try:
            if path.is_file() and path.stat().st_mtime >= cutoff:
                result.append(path)
        except OSError:
            continue
    return result


def load_jsonl(path: Path) -> Tuple[List[Mapping[str, Any]], int]:
    records: List[Mapping[str, Any]] = []
    malformed = 0
    try:
        with path.open("r", encoding="utf-8") as handle:
            for line in handle:
                try:
                    value = json.loads(line)
                except (json.JSONDecodeError, UnicodeDecodeError):
                    malformed += 1
                    continue
                if isinstance(value, dict):
                    records.append(value)
    except OSError:
        pass
    return records, malformed


def load_json_stream(path: Path) -> Tuple[List[Mapping[str, Any]], int]:
    """Read JSONL or whitespace-separated pretty-printed JSON objects."""
    try:
        content = path.read_text(encoding="utf-8")
    except OSError:
        return [], 0
    decoder = json.JSONDecoder()
    values: List[Mapping[str, Any]] = []
    malformed = 0
    offset = 0
    while offset < len(content):
        while offset < len(content) and content[offset].isspace():
            offset += 1
        if offset >= len(content):
            break
        try:
            value, offset = decoder.raw_decode(content, offset)
        except json.JSONDecodeError:
            malformed += 1
            break
        if isinstance(value, dict):
            values.append(value)
    return values, malformed


def aggregate_codex(window: Window, root: Path, resolver: RepoResolver) -> AdapterOutput:
    files = recent_files(root, "*.jsonl", window.start)
    output: List[UsageRecord] = []
    malformed = 0
    event_count = 0
    mismatches = 0
    for path in files:
        rows, errors = load_jsonl(path)
        malformed += errors
        meta: Mapping[str, Any] = {}
        events: List[Tuple[datetime, Mapping[str, Any], Mapping[str, Any]]] = []
        for row in rows:
            if row.get("type") == "session_meta" and not meta:
                payload = row.get("payload")
                meta = payload if isinstance(payload, dict) else {}
                continue
            payload = row.get("payload")
            if row.get("type") != "event_msg" or not isinstance(payload, dict):
                continue
            if payload.get("type") != "token_count":
                continue
            occurred = parse_time(row.get("timestamp"))
            info = payload.get("info")
            if occurred is None or not isinstance(info, dict):
                continue
            usage = info.get("total_token_usage")
            last = info.get("last_token_usage")
            if isinstance(usage, dict) and integer(usage.get("total_tokens")) is not None:
                events.append((occurred, usage, last if isinstance(last, dict) else {}))

        previous = 0
        consumed = 0
        cumulative_delta = 0
        last_usage_sum = 0
        last_usage_present = False
        for occurred, usage, last in sorted(events, key=lambda item: item[0]):
            current = integer(usage.get("total_tokens")) or 0
            delta = current - previous if current >= previous else current
            if window.contains(occurred):
                cumulative_delta += delta
                last_total = integer(last.get("total_tokens"))
                if last_total is None:
                    consumed += delta
                else:
                    consumed += last_total
                    last_usage_sum += last_total
                    last_usage_present = True
                event_count += 1
            previous = current
        if not consumed:
            continue
        if last_usage_present and last_usage_sum != cumulative_delta:
            mismatches += 1
        git_meta = meta.get("git") if isinstance(meta.get("git"), dict) else {}
        repo = resolver.resolve(
            cwd=meta.get("cwd"),
            source_path=path,
            remote=git_meta.get("repository_url"),
        )
        session = str(meta.get("id") or meta.get("session_id") or path.stem)
        provider = normalize_provider(meta.get("model_provider")) or "openai"
        output.append(
            UsageRecord(
                source="codex",
                tokens=consumed,
                timestamp=window.end,
                repo=repo,
                provider=provider,
                session=session,
            )
        )
    return AdapterOutput(
        output,
        {
            "available": root.exists(),
            "files_scanned": len(files),
            "events": event_count,
            "malformed_records": malformed,
            "sessions_where_deltas_differ_from_last_usage": mismatches,
            "method": "sum last_token_usage per call; cumulative positive delta fallback",
        },
    )


def aggregate_claude(window: Window, root: Path, resolver: RepoResolver) -> AdapterOutput:
    files = recent_files(root, "*.jsonl", window.start)
    candidates: Dict[str, UsageRecord] = {}
    malformed = 0
    duplicates = 0
    for path in files:
        rows, errors = load_jsonl(path)
        malformed += errors
        for row in rows:
            if row.get("type") != "assistant":
                continue
            occurred = parse_time(row.get("timestamp"))
            if occurred is None or not window.contains(occurred):
                continue
            message = row.get("message")
            if not isinstance(message, dict):
                continue
            usage = message.get("usage")
            if not isinstance(usage, dict):
                continue
            event_id = str(message.get("id") or row.get("uuid") or "")
            if not event_id:
                continue
            tokens = sum(
                integer(usage.get(key)) or 0
                for key in (
                    "input_tokens",
                    "cache_creation_input_tokens",
                    "cache_read_input_tokens",
                    "output_tokens",
                )
            )
            if tokens <= 0:
                continue
            candidate = UsageRecord(
                source="claude_code",
                tokens=tokens,
                timestamp=occurred,
                repo=resolver.resolve(row.get("cwd"), path),
                provider="anthropic",
                model=message.get("model") if isinstance(message.get("model"), str) else None,
                session=str(row.get("sessionId") or path.stem),
                event_id=event_id,
            )
            existing = candidates.get(event_id)
            if existing is not None:
                duplicates += 1
            if existing is None or (candidate.tokens, candidate.timestamp) > (
                existing.tokens,
                existing.timestamp,
            ):
                candidates[event_id] = candidate
    return AdapterOutput(
        list(candidates.values()),
        {
            "available": root.exists(),
            "files_scanned": len(files),
            "unique_messages": len(candidates),
            "duplicate_or_partial_records": duplicates,
            "malformed_records": malformed,
            "method": "max final usage per globally unique message id",
        },
    )


def aggregate_grok(window: Window, root: Path, resolver: RepoResolver) -> AdapterOutput:
    log_path = root / "logs" / "unified.jsonl"
    session_root = root / "sessions"
    summary_files = recent_files(session_root, "summary.json", window.start)
    session_info: Dict[str, Tuple[Optional[str], Optional[str]]] = {}
    session_intervals: List[Tuple[datetime, datetime]] = []
    malformed_summaries = 0

    for path in summary_files:
        try:
            summary = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError):
            malformed_summaries += 1
            continue
        if not isinstance(summary, dict):
            continue
        session = path.parent.name
        encoded_cwd = path.parent.parent.name
        cwd = summary.get("git_root_dir") or unquote(encoded_cwd)
        repo = resolver.resolve(cwd, path)
        model = (
            summary.get("current_model_id")
            if isinstance(summary.get("current_model_id"), str)
            else None
        )
        session_info[session] = (repo, model)

        created = parse_time(summary.get("created_at"))
        updated = parse_time(summary.get("updated_at")) or created
        if created and updated and created <= window.end and updated >= window.start:
            session_intervals.append((created, updated))

    rows, malformed_logs = load_jsonl(log_path) if log_path.exists() else ([], 0)
    output: List[UsageRecord] = []
    seen: set[Tuple[str, str, int, int]] = set()
    duplicates = 0
    first_log_time: Optional[datetime] = None
    for row in rows:
        occurred = parse_time(row.get("ts"))
        if occurred is not None and (first_log_time is None or occurred < first_log_time):
            first_log_time = occurred
        if row.get("msg") != "shell.turn.inference_done":
            continue
        context = row.get("ctx")
        if occurred is None or not window.contains(occurred) or not isinstance(context, dict):
            continue
        prompt = integer(context.get("prompt_tokens"))
        completion = integer(context.get("completion_tokens"))
        if prompt is None:
            prompt = integer(context.get("cached_prompt_tokens"))
        if completion is None:
            completion = integer(context.get("reasoning_tokens"))
        if prompt is None and completion is None:
            continue
        prompt = max(prompt or 0, 0)
        completion = max(completion or 0, 0)
        total = prompt + completion
        if total <= 0:
            continue
        session = str(row.get("sid") or "")
        event_time = str(row.get("ts") or "")
        dedupe_key = (session, event_time, prompt, completion)
        if dedupe_key in seen:
            duplicates += 1
            continue
        seen.add(dedupe_key)
        repo, model = session_info.get(session, (None, None))
        output.append(
            UsageRecord(
                source="grok",
                tokens=total,
                timestamp=occurred,
                repo=repo,
                provider="xai",
                model=model,
                session=session or None,
                event_id=f"{session}:{event_time}",
            )
        )

    partial_history = bool(
        first_log_time
        and any(updated < first_log_time for _, updated in session_intervals)
    )
    return AdapterOutput(
        output,
        {
            "available": root.exists(),
            "files_scanned": 1 if log_path.exists() else 0,
            "session_summaries": len(summary_files),
            "inference_events": len(output),
            "duplicate_records": duplicates,
            "malformed_records": malformed_logs + malformed_summaries,
            "first_log_timestamp": first_log_time.isoformat() if first_log_time else None,
            "partial_history": partial_history,
            "method": "sum prompt and completion tokens per completed inference",
        },
    )


def aggregate_pi(window: Window, root: Path, resolver: RepoResolver) -> AdapterOutput:
    files = recent_files(root, "*.jsonl", window.start)
    output: List[UsageRecord] = []
    seen: set[str] = set()
    malformed = 0
    duplicates = 0
    for path in files:
        rows, errors = load_jsonl(path)
        malformed += errors
        session_meta = next((row for row in rows if row.get("type") == "session"), {})
        cwd = session_meta.get("cwd") if isinstance(session_meta, dict) else None
        session = (
            str(session_meta.get("id") or path.stem)
            if isinstance(session_meta, dict)
            else path.stem
        )
        repo = resolver.resolve(cwd, path)
        for row in rows:
            message = row.get("message")
            if row.get("type") != "message" or not isinstance(message, dict):
                continue
            if message.get("role") != "assistant":
                continue
            occurred = parse_time(row.get("timestamp"))
            usage = message.get("usage")
            if occurred is None or not window.contains(occurred) or not isinstance(usage, dict):
                continue
            event_id = str(row.get("id") or message.get("responseId") or "")
            dedupe_key = f"{session}:{event_id}"
            if not event_id or dedupe_key in seen:
                duplicates += 1
                continue
            seen.add(dedupe_key)
            total = integer(usage.get("totalTokens"))
            if total is None:
                total = sum(
                    integer(usage.get(key)) or 0
                    for key in ("input", "output", "cacheRead", "cacheWrite")
                )
            if total <= 0:
                continue
            model = message.get("model") if isinstance(message.get("model"), str) else None
            output.append(
                UsageRecord(
                    source="pi",
                    tokens=total,
                    timestamp=occurred,
                    repo=repo,
                    provider=normalize_provider(message.get("provider"), model),
                    model=model,
                    session=session,
                    event_id=event_id,
                )
            )
    return AdapterOutput(
        output,
        {
            "available": root.exists(),
            "files_scanned": len(files),
            "messages": len(output),
            "duplicate_records": duplicates,
            "malformed_records": malformed,
            "method": "sum persisted assistant-message usage",
        },
    )


def aggregate_opencode(window: Window, root: Path, resolver: RepoResolver) -> AdapterOutput:
    sessions: Dict[str, Mapping[str, Any]] = {}
    session_root = root / "session"
    if session_root.exists():
        for path in session_root.rglob("*.json"):
            try:
                value = json.loads(path.read_text(encoding="utf-8"))
            except (OSError, UnicodeDecodeError, json.JSONDecodeError):
                continue
            if isinstance(value, dict) and isinstance(value.get("id"), str):
                sessions[value["id"]] = value

    message_root = root / "message"
    files = recent_files(message_root, "*.json", window.start)
    output: List[UsageRecord] = []
    malformed = 0
    seen: set[str] = set()
    duplicates = 0
    for path in files:
        try:
            row = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError):
            malformed += 1
            continue
        if not isinstance(row, dict) or row.get("role") != "assistant":
            continue
        event_id = str(row.get("id") or path.stem)
        if event_id in seen:
            duplicates += 1
            continue
        seen.add(event_id)
        time_data = row.get("time") if isinstance(row.get("time"), dict) else {}
        occurred = parse_time(time_data.get("completed") or time_data.get("created"))
        if occurred is None or not window.contains(occurred):
            continue
        tokens_data = row.get("tokens") if isinstance(row.get("tokens"), dict) else {}
        cache = tokens_data.get("cache") if isinstance(tokens_data.get("cache"), dict) else {}
        total = sum(
            integer(value) or 0
            for value in (
                tokens_data.get("input"),
                tokens_data.get("output"),
                tokens_data.get("reasoning"),
                cache.get("read"),
                cache.get("write"),
            )
        )
        if total <= 0:
            continue
        session_id = str(row.get("sessionID") or "")
        session = sessions.get(session_id, {})
        directory = session.get("directory") if isinstance(session, dict) else None
        model = row.get("modelID") if isinstance(row.get("modelID"), str) else None
        output.append(
            UsageRecord(
                source="opencode",
                tokens=total,
                timestamp=occurred,
                repo=resolver.resolve(directory, path),
                provider=normalize_provider(row.get("providerID"), model),
                model=model,
                session=session_id or None,
                event_id=event_id,
            )
        )
    return AdapterOutput(
        output,
        {
            "available": root.exists(),
            "files_scanned": len(files),
            "sessions_indexed": len(sessions),
            "messages": len(output),
            "duplicate_records": duplicates,
            "malformed_records": malformed,
            "method": "sum input, output, reasoning, and cache tokens per assistant message",
        },
    )


def candidate_repo_path(row: Mapping[str, Any]) -> Any:
    for key in ("cwd", "directory", "repo_path", "worktree", "repository", "repo", "env_path"):
        value = row.get(key)
        if isinstance(value, str) and value:
            return value
    workspace = row.get("workspace")
    if isinstance(workspace, dict):
        for key in ("path", "cwd", "directory", "repo_path", "worktree"):
            value = workspace.get(key)
            if isinstance(value, str) and value:
                return value
    return None


def metric_tokens(metrics: Any) -> Optional[int]:
    if not isinstance(metrics, dict):
        return None
    for key in ("totalTokens", "total_tokens", "tokens"):
        total = integer(metrics.get(key))
        if total is not None:
            return max(total, 0)
    keys = (
        "inputTokens",
        "outputTokens",
        "totalInputTokens",
        "totalOutputTokens",
        "cacheReadTokens",
        "cacheWriteTokens",
        "input_tokens",
        "output_tokens",
        "cache_read_tokens",
        "cache_write_tokens",
    )
    values = [integer(metrics.get(key)) for key in keys]
    if not any(value is not None for value in values):
        return None
    return sum(value or 0 for value in values)


def aggregate_crush(window: Window, root: Path, resolver: RepoResolver) -> AdapterOutput:
    files = recent_files(root, "*.json", window.start, recursive=False)
    output: List[UsageRecord] = []
    malformed = 0
    records_without_tokens = 0
    for path in files:
        try:
            row = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError):
            malformed += 1
            continue
        if not isinstance(row, dict):
            continue
        repo = resolver.resolve(candidate_repo_path(row), path)
        session = str(row.get("id") or path.stem)
        default_time = parse_time(row.get("updatedAt") or row.get("createdAt"))
        cells = row.get("cells") if isinstance(row.get("cells"), list) else []
        emitted = False
        for index, cell in enumerate(cells):
            if not isinstance(cell, dict) or cell.get("type") not in (None, "CELL_TYPE_AGENT"):
                continue
            total = metric_tokens(cell.get("metrics"))
            occurred = parse_time(cell.get("updatedAt") or cell.get("createdAt")) or default_time
            if total is None or total <= 0 or occurred is None or not window.contains(occurred):
                continue
            agent_output = (
                cell.get("agentOutput") if isinstance(cell.get("agentOutput"), dict) else {}
            )
            model = (
                agent_output.get("model")
                if isinstance(agent_output.get("model"), str)
                else None
            )
            output.append(
                UsageRecord(
                    source="crush",
                    tokens=total,
                    timestamp=occurred,
                    repo=repo,
                    provider=normalize_provider(agent_output.get("provider"), model),
                    model=model,
                    session=session,
                    event_id=str(cell.get("id") or f"{session}:{index}"),
                )
            )
            emitted = True
        if emitted:
            continue
        total = metric_tokens(row.get("metrics"))
        if total is None:
            records_without_tokens += 1
            continue
        if total > 0 and default_time is not None and window.contains(default_time):
            model = row.get("model") if isinstance(row.get("model"), str) else None
            output.append(
                UsageRecord(
                    source="crush",
                    tokens=total,
                    timestamp=default_time,
                    repo=repo,
                    provider=normalize_provider(row.get("provider"), model),
                    model=model,
                    session=session,
                )
            )
    return AdapterOutput(
        output,
        {
            "available": root.exists(),
            "files_scanned": len(files),
            "records_without_tokens": records_without_tokens,
            "malformed_records": malformed,
            "method": "sum persisted agent-cell metrics when repository metadata exists",
        },
    )


def aggregate_squire(window: Window, path: Path, resolver: RepoResolver) -> AdapterOutput:
    rows, malformed = load_json_stream(path) if path.exists() else ([], 0)
    output: List[UsageRecord] = []
    records_without_tokens = 0
    for row in rows:
        metrics = row.get("metrics") if isinstance(row.get("metrics"), dict) else row
        total = metric_tokens(metrics)
        if total is None:
            records_without_tokens += 1
            continue
        occurred = parse_time(
            row.get("completed_at")
            or row.get("updated_at")
            or row.get("started_at")
            or row.get("created_at")
        )
        if total <= 0 or occurred is None or not window.contains(occurred):
            continue
        model = row.get("model") if isinstance(row.get("model"), str) else None
        repo_value = candidate_repo_path(row)
        output.append(
            UsageRecord(
                source="squire",
                tokens=total,
                timestamp=occurred,
                repo=resolver.resolve(repo_value, path),
                provider=normalize_provider(row.get("provider"), model),
                model=model,
                session=str(row.get("task_id") or row.get("env_id") or "") or None,
            )
        )
    return AdapterOutput(
        output,
        {
            "available": path.exists(),
            "files_scanned": 1 if path.exists() else 0,
            "records": len(rows),
            "records_without_tokens": records_without_tokens,
            "malformed_records": malformed,
            "method": "sum local Squire metrics only when token and repository fields are present",
        },
    )


def inspect_cursor(root: Path) -> AdapterOutput:
    transcripts = list(root.glob("*/agent-transcripts/*/*.jsonl")) if root.exists() else []
    return AdapterOutput(
        [],
        {
            "available": root.exists(),
            "files_scanned": len(transcripts),
            "records_without_tokens": len(transcripts),
            "status_hint": "no_token_fields",
            "method": "transcripts contain messages but no local token counters",
        },
    )


def source_status(diagnostics: Mapping[str, Any], records: Sequence[UsageRecord]) -> str:
    if not diagnostics.get("available"):
        return "not_found"
    if any(record.repo and record.tokens > 0 for record in records):
        return "included"
    if any(not record.repo and record.tokens > 0 for record in records):
        return "unattributed"
    if diagnostics.get("status_hint") == "no_token_fields" or diagnostics.get(
        "records_without_tokens", 0
    ):
        return "no_token_fields"
    return "no_usage_in_window"


def collect_report(window: Window, stores: StorePaths) -> Dict[str, Any]:
    resolver = RepoResolver(stores.repo_home)
    adapters = {
        "codex": aggregate_codex(window, stores.codex, resolver),
        "claude_code": aggregate_claude(window, stores.claude, resolver),
        "grok": aggregate_grok(window, stores.grok, resolver),
        "pi": aggregate_pi(window, stores.pi, resolver),
        "opencode": aggregate_opencode(window, stores.opencode, resolver),
        "crush": aggregate_crush(window, stores.crush, resolver),
        "squire": aggregate_squire(window, stores.squire_metrics, resolver),
        "cursor": inspect_cursor(stores.cursor),
    }

    repo_rows: Dict[str, Dict[str, Any]] = {}
    session_sets: Dict[str, Dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
    source_totals: Dict[str, int] = defaultdict(int)
    provider_totals: Dict[str, int] = defaultdict(int)
    unattributed_tokens = 0
    diagnostics: Dict[str, Dict[str, Any]] = {}

    for source in SOURCE_ORDER:
        adapter = adapters[source]
        attributed = sum(record.tokens for record in adapter.records if record.repo)
        unattributed = sum(record.tokens for record in adapter.records if not record.repo)
        detail = dict(adapter.diagnostics)
        detail.update(
            {
                "status": source_status(detail, adapter.records),
                "included_tokens": attributed,
                "unattributed_tokens": unattributed,
                "usage_records": len(adapter.records),
            }
        )
        diagnostics[source] = detail
        unattributed_tokens += unattributed

        for record in adapter.records:
            if not record.repo:
                continue
            row = repo_rows.setdefault(
                record.repo,
                {
                    "tokens": 0,
                    "sources": defaultdict(int),
                    "providers": defaultdict(int),
                    "models": defaultdict(int),
                },
            )
            row["tokens"] += record.tokens
            row["sources"][record.source] += record.tokens
            source_totals[record.source] += record.tokens
            if record.provider:
                row["providers"][record.provider] += record.tokens
                provider_totals[record.provider] += record.tokens
            if record.model:
                row["models"][record.model] += record.tokens
            if record.session:
                session_sets[record.repo][record.source].add(record.session)

    ordered_repos: Dict[str, Dict[str, Any]] = {}
    for repo, row in sorted(repo_rows.items(), key=lambda item: item[1]["tokens"], reverse=True):
        ordered_repos[repo] = {
            "tokens": row["tokens"],
            "sources": dict(sorted(row["sources"].items())),
            "providers": dict(sorted(row["providers"].items())),
            "models": dict(sorted(row["models"].items())),
            "sessions": {
                source: len(sessions)
                for source, sessions in sorted(session_sets[repo].items())
            },
        }

    return {
        "window": {
            "start": window.start.isoformat(),
            "end": window.end.isoformat(),
            "timezone": window.end.tzname(),
        },
        "method": "sum provider-reported tokens, including cache and reasoning where recorded",
        "repos": ordered_repos,
        "source_totals": dict(sorted(source_totals.items())),
        "provider_totals": dict(sorted(provider_totals.items())),
        "unattributed_tokens": unattributed_tokens,
        "diagnostics": diagnostics,
    }


def comma(value: int) -> str:
    return f"{value:,}"


def display_sources(report: Mapping[str, Any]) -> List[str]:
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, dict):
        return []
    names: List[str] = []
    for source in SOURCE_ORDER:
        detail = diagnostics.get(source)
        if not isinstance(detail, dict):
            continue
        available = detail.get("available", detail.get("status") != "not_found")
        if not available:
            continue
        if source in ("codex", "claude_code", "grok", "pi", "opencode") or detail.get(
            "included_tokens", 0
        ):
            names.append(SOURCE_NAMES[source])
    return names


def telemetry_gaps(report: Mapping[str, Any]) -> List[str]:
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, dict):
        return []
    gaps: List[str] = []
    grok = diagnostics.get("grok")
    if isinstance(grok, dict) and grok.get("partial_history"):
        gaps.append("Grok local history partial")
    squire = diagnostics.get("squire")
    if isinstance(squire, dict) and squire.get("status") == "no_token_fields":
        gaps.append("Squire no token totals")
    crush = diagnostics.get("crush")
    if isinstance(crush, dict) and crush.get("status") == "unattributed":
        gaps.append("Crush lacks repo attribution")
    cursor = diagnostics.get("cursor")
    if isinstance(cursor, dict) and cursor.get("status") == "no_token_fields":
        gaps.append("Cursor no counters")
    return gaps


def render_scorecard(report: Mapping[str, Any], top: int = 5) -> str:
    repos = report.get("repos")
    if not isinstance(repos, dict):
        repos = {}
    ordered = list(repos.items())[: max(top, 1)]
    attributed_total = sum(
        int(row.get("tokens", 0)) for row in repos.values() if isinstance(row, dict)
    )
    top_total = sum(int(row.get("tokens", 0)) for _, row in ordered if isinstance(row, dict))
    coverage = (100.0 * top_total / attributed_total) if attributed_total else 0.0
    top_name, top_tokens = ("n/a", 0)
    if ordered and isinstance(ordered[0][1], dict):
        top_name = ordered[0][0]
        top_tokens = int(ordered[0][1].get("tokens", 0))
    top_share = (100.0 * top_tokens / attributed_total) if attributed_total else 0.0
    pair_total = sum(
        int(row.get("tokens", 0)) for _, row in ordered[:2] if isinstance(row, dict)
    )
    pair_share = (100.0 * pair_total / attributed_total) if attributed_total else 0.0

    window_data = report.get("window") if isinstance(report.get("window"), dict) else {}
    start = parse_time(window_data.get("start"))
    end = parse_time(window_data.get("end"))
    if start and end:
        zone = str(window_data.get("timezone") or end.tzname() or "local")
        subtitle = (
            "7-day local window · "
            f"{start.strftime('%b %-d %H:%M')} → {end.strftime('%b %-d %H:%M')} {zone}"
        )
        generated = end.strftime("%Y-%m-%d %H:%M ") + zone
    else:
        subtitle = "7-day local window"
        generated = "unknown time"

    sources = display_sources(report)
    source_text = " + ".join(sources) if sources else "no local token stores"
    diagnostics = report.get("diagnostics")
    active_sources = 0
    if isinstance(diagnostics, dict):
        active_sources = sum(
            1
            for detail in diagnostics.values()
            if isinstance(detail, dict) and detail.get("included_tokens", 0)
        )
    gaps = telemetry_gaps(report)
    lines = [
        "# Weekly agent-token footprint",
        f"sub: {subtitle}",
        f"meta: {len(sources)} supported · {active_sources} active · "
        f"top {len(ordered)} · {coverage:.1f}% coverage",
        f"footer: cached context included · Grok/xAI counted when logged · {generated}",
        "groups: header=100, chart=100, callouts=90, titles=50",
        "",
        "## Chart: Tokens consumed per repository (millions)",
        "type: histogram",
        "| repository | tokens (M) |",
        "| --- | ---: |",
    ]
    for repo, row in ordered:
        tokens = int(row.get("tokens", 0)) if isinstance(row, dict) else 0
        lines.append(f"| {repo} | {tokens / 1_000_000:.1f} |")
    lines.extend(
        [
            "",
            "## Callouts",
            f"| TOP | {top_name} · {comma(top_tokens)} tokens · {top_share:.1f}% of attributed |",
            f"| CONCENTRATION | top two · {pair_share:.1f}% of attributed |",
            f"| COVERAGE | top {len(ordered)} · {comma(top_total)} tokens · {coverage:.1f}% |",
            f"| SOURCES | {source_text} |",
        ]
    )
    if gaps:
        lines.append(f"| GAPS | {' · '.join(gaps)} |")
    unattributed = int(report.get("unattributed_tokens", 0) or 0)
    if unattributed:
        lines.append(f"| UNATTRIBUTED | {comma(unattributed)} tokens omitted from repo bars |")
    return "\n".join(lines) + "\n"


def local_window(days: int, now: Optional[datetime] = None) -> Window:
    if days < 1:
        raise ValueError("days must be positive")
    current = now or datetime.now().astimezone()
    if current.tzinfo is None:
        current = current.replace(tzinfo=timezone.utc)
    start = current.replace(hour=0, minute=0, second=0, microsecond=0) - timedelta(days=days - 1)
    return Window(start, current)


def parse_cli_time(value: str) -> datetime:
    parsed = parse_time(value)
    if parsed is None:
        raise argparse.ArgumentTypeError(f"invalid ISO-8601 timestamp: {value}")
    return parsed


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--start", type=parse_cli_time, help="inclusive ISO-8601 start")
    parser.add_argument("--end", type=parse_cli_time, help="inclusive ISO-8601 end")
    parser.add_argument("--days", type=int, default=7, help="local calendar days (default: 7)")
    parser.add_argument("--format", choices=("json", "scorecard"), default="json")
    parser.add_argument("--top", type=int, default=5, help="repository rows in scorecard output")
    parser.add_argument("--home", type=Path, default=Path.home())
    parser.add_argument("--repo-home", type=Path)
    args = parser.parse_args(argv)

    if bool(args.start) != bool(args.end):
        parser.error("--start and --end must be provided together")
    try:
        window = (
            Window(args.start, args.end)
            if args.start and args.end
            else local_window(args.days)
        )
    except ValueError as error:
        parser.error(str(error))
    stores = default_store_paths(args.home, args.repo_home)
    report = collect_report(window, stores)
    if args.format == "scorecard":
        print(render_scorecard(report, args.top), end="")
    else:
        print(json.dumps(report, indent=2, sort_keys=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
