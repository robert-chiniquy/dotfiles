#!/usr/bin/env python3

import json
import sys
import tempfile
import unittest
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import quote

sys.path.insert(0, str(Path(__file__).resolve().parent))

import weekly_repo_tokens as wrt


START = datetime(2026, 8, 1, tzinfo=timezone.utc)
END = datetime(2026, 8, 4, tzinfo=timezone.utc)


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding="utf-8")


def write_jsonl(path: Path, values: list[object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "".join(json.dumps(value) + "\n" for value in values),
        encoding="utf-8",
    )


class WeeklyRepoTokensTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        self.repo_home = self.root / "repo"
        (self.repo_home / "occult").mkdir(parents=True)
        self.stores = wrt.StorePaths(
            repo_home=self.repo_home,
            codex=self.root / "codex",
            claude=self.root / "claude",
            grok=self.root / "grok",
            pi=self.root / "pi",
            opencode=self.root / "opencode",
            crush=self.root / "crush",
            cursor=self.root / "cursor",
            squire_metrics=self.root / "squire-metrics.jsonl",
        )
        self.window = wrt.Window(START, END)

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def test_collects_provider_agnostic_usage_without_double_counting(self) -> None:
        write_jsonl(
            self.stores.codex / "2026" / "08" / "session.jsonl",
            [
                {
                    "type": "session_meta",
                    "payload": {
                        "id": "codex-session",
                        "cwd": str(self.repo_home / "occult"),
                        "model_provider": "openai",
                        "git": {"repository_url": "git@github.com:ConductorOne/occult.git"},
                    },
                },
                {
                    "type": "event_msg",
                    "timestamp": "2026-08-02T10:00:00Z",
                    "payload": {
                        "type": "token_count",
                        "info": {
                            "total_token_usage": {"total_tokens": 100},
                            "last_token_usage": {"total_tokens": 100},
                        },
                    },
                },
                {
                    "type": "event_msg",
                    "timestamp": "2026-08-02T10:01:00Z",
                    "payload": {
                        "type": "token_count",
                        "info": {
                            "total_token_usage": {"total_tokens": 160},
                            "last_token_usage": {"total_tokens": 60},
                        },
                    },
                },
            ],
        )

        claude_message = {
            "type": "assistant",
            "timestamp": "2026-08-02T11:00:00Z",
            "cwd": str(self.repo_home / "occult"),
            "sessionId": "claude-session",
            "message": {
                "id": "shared-message",
                "model": "claude-opus-5",
                "usage": {
                    "input_tokens": 1,
                    "cache_creation_input_tokens": 0,
                    "cache_read_input_tokens": 20,
                    "output_tokens": 4,
                },
            },
        }
        write_jsonl(self.stores.claude / "project-a" / "partial.jsonl", [claude_message])
        write_jsonl(
            self.stores.claude / "project-a" / "duplicate.jsonl",
            [
                {
                    **claude_message,
                    "message": {
                        **claude_message["message"],
                        "usage": {
                            "input_tokens": 1,
                            "cache_creation_input_tokens": 0,
                            "cache_read_input_tokens": 5,
                            "output_tokens": 4,
                        },
                    },
                }
            ],
        )

        grok_session = "grok-session"
        grok_cwd = quote(str(self.repo_home / "occult"), safe="")
        write_json(
            self.stores.grok / "sessions" / grok_cwd / grok_session / "summary.json",
            {
                "created_at": "2026-08-02T11:30:00Z",
                "updated_at": "2026-08-02T11:31:00Z",
                "current_model_id": "grok-4-fast",
                "git_root_dir": str(self.repo_home / "occult"),
            },
        )
        grok_usage = {
            "ts": "2026-08-02T11:31:00Z",
            "sid": grok_session,
            "src": "shell",
            "msg": "shell.turn.inference_done",
            "ctx": {
                "prompt_tokens": 17,
                "cached_prompt_tokens": 13,
                "completion_tokens": 3,
                "reasoning_tokens": 2,
            },
        }
        write_jsonl(
            self.stores.grok / "logs" / "unified.jsonl",
            [grok_usage, grok_usage],
        )

        write_jsonl(
            self.stores.pi / "--Users-test-repo-occult--" / "session.jsonl",
            [
                {
                    "type": "session",
                    "id": "pi-session",
                    "timestamp": "2026-08-02T12:00:00Z",
                    "cwd": str(self.repo_home / "occult"),
                },
                {
                    "type": "message",
                    "id": "pi-response",
                    "timestamp": "2026-08-02T12:01:00Z",
                    "message": {
                        "role": "assistant",
                        "provider": "xai",
                        "model": "grok-code-fast-1",
                        "usage": {"totalTokens": 33},
                    },
                },
            ],
        )

        session_id = "ses_grok"
        write_json(
            self.stores.opencode / "session" / "project" / f"{session_id}.json",
            {
                "id": session_id,
                "directory": str(self.repo_home / "occult"),
                "time": {"created": 1785650000000},
            },
        )
        write_json(
            self.stores.opencode / "message" / session_id / "msg_grok.json",
            {
                "id": "msg_grok",
                "sessionID": session_id,
                "role": "assistant",
                "providerID": "xai",
                "modelID": "grok-4",
                "time": {"created": 1785650100000},
                "tokens": {
                    "input": 2,
                    "output": 3,
                    "reasoning": 4,
                    "cache": {"read": 5, "write": 6},
                },
            },
        )

        write_json(
            self.stores.crush / "attributed.json",
            {
                "id": "crush-session",
                "createdAt": "2026-08-02T13:00:00Z",
                "cwd": str(self.repo_home / "occult"),
                "cells": [
                    {
                        "type": "CELL_TYPE_AGENT",
                        "agentOutput": {"model": "gemini-2.5-pro"},
                        "metrics": {"inputTokens": 7, "outputTokens": 8},
                    }
                ],
            },
        )
        write_json(
            self.stores.crush / "unattributed.json",
            {
                "id": "crush-unattributed",
                "createdAt": "2026-08-02T13:30:00Z",
                "metrics": {"totalInputTokens": 40, "totalOutputTokens": 60},
                "cells": [],
            },
        )

        write_jsonl(
            self.stores.squire_metrics,
            [
                {
                    "env_id": "squire-env",
                    "completed_at": "2026-08-02T14:00:00Z",
                    "repo_path": str(self.repo_home / "occult"),
                    "provider": "anthropic",
                    "model": "claude-opus-5",
                    "total_tokens": 12,
                }
            ],
        )
        self.stores.cursor.mkdir()

        report = wrt.collect_report(self.window, self.stores)
        occult = report["repos"]["occult"]

        self.assertEqual(occult["tokens"], 285)
        self.assertEqual(occult["sources"]["codex"], 160)
        self.assertEqual(occult["sources"]["claude_code"], 25)
        self.assertEqual(occult["sources"]["grok"], 20)
        self.assertEqual(occult["sources"]["pi"], 33)
        self.assertEqual(occult["sources"]["opencode"], 20)
        self.assertEqual(occult["sources"]["crush"], 15)
        self.assertEqual(occult["sources"]["squire"], 12)
        self.assertEqual(occult["providers"]["xai"], 73)
        self.assertEqual(occult["models"]["grok-4-fast"], 20)
        self.assertEqual(occult["models"]["grok-4"], 20)
        self.assertEqual(occult["models"]["grok-code-fast-1"], 33)
        self.assertEqual(report["unattributed_tokens"], 100)
        self.assertEqual(
            report["diagnostics"]["claude_code"]["duplicate_or_partial_records"],
            1,
        )
        self.assertEqual(report["diagnostics"]["grok"]["duplicate_records"], 1)
        self.assertFalse(report["diagnostics"]["grok"]["partial_history"])
        self.assertEqual(report["diagnostics"]["cursor"]["status"], "no_token_fields")

    def test_squire_without_token_fields_is_an_explicit_gap(self) -> None:
        write_jsonl(
            self.stores.squire_metrics,
            [
                {
                    "env_id": "legacy-env",
                    "completed_at": "2026-08-02T14:00:00Z",
                    "duration_seconds": 20,
                    "commit_count": 1,
                }
            ],
        )

        report = wrt.collect_report(self.window, self.stores)

        self.assertEqual(report["diagnostics"]["squire"]["status"], "no_token_fields")
        self.assertEqual(report["diagnostics"]["squire"]["records_without_tokens"], 1)

    def test_grok_reports_when_local_log_misses_earlier_sessions(self) -> None:
        grok_session = "older-grok-session"
        grok_cwd = quote(str(self.repo_home / "occult"), safe="")
        write_json(
            self.stores.grok / "sessions" / grok_cwd / grok_session / "summary.json",
            {
                "created_at": "2026-08-01T08:00:00Z",
                "updated_at": "2026-08-01T09:00:00Z",
                "git_root_dir": str(self.repo_home / "occult"),
            },
        )
        write_jsonl(
            self.stores.grok / "logs" / "unified.jsonl",
            [
                {
                    "ts": "2026-08-02T10:00:00Z",
                    "sid": "later-session",
                    "msg": "shell.session.started",
                    "ctx": {},
                }
            ],
        )

        report = wrt.collect_report(self.window, self.stores)

        self.assertTrue(report["diagnostics"]["grok"]["partial_history"])

    def test_model_family_recognizes_grok_and_other_common_providers(self) -> None:
        self.assertEqual(wrt.provider_from_model("grok-4"), "xai")
        self.assertEqual(wrt.provider_from_model("claude-opus-5"), "anthropic")
        self.assertEqual(wrt.provider_from_model("gemini-2.5-pro"), "google")
        self.assertEqual(wrt.provider_from_model("gpt-5.4"), "openai")
        self.assertEqual(wrt.provider_from_model("qwen2.5:14b"), "local")

    def test_scorecard_names_included_sources_and_telemetry_gaps(self) -> None:
        report = {
            "window": {"start": START.isoformat(), "end": END.isoformat()},
            "repos": {
                "occult": {
                    "tokens": 2_000_000,
                    "sources": {"codex": 1_000_000, "opencode": 1_000_000},
                    "providers": {"openai": 1_000_000, "xai": 1_000_000},
                    "models": {"gpt-5.4": 1_000_000, "grok-4": 1_000_000},
                }
            },
            "diagnostics": {
                "codex": {"status": "included", "included_tokens": 1_000_000},
                "grok": {"status": "included", "included_tokens": 1_000_000},
                "opencode": {"status": "included", "included_tokens": 1_000_000},
                "squire": {"status": "no_token_fields", "included_tokens": 0},
                "crush": {"status": "unattributed", "included_tokens": 0},
            },
            "unattributed_tokens": 10,
        }

        rendered = wrt.render_scorecard(report, top=5)

        self.assertIn("Codex + Grok + OpenCode", rendered)
        self.assertIn("Grok/xAI counted when logged", rendered)
        self.assertIn("Squire no token totals", rendered)
        self.assertIn("Crush lacks repo attribution", rendered)


if __name__ == "__main__":
    unittest.main()
