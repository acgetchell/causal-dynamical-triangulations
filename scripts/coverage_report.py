#!/usr/bin/env python3
"""
Utility for summarizing Tarpaulin coverage results.

The script expects a JSON report produced by `cargo tarpaulin --out Json`
(Default location: `tarpaulin-report.json`). It prints all files that have
coverable lines, sorted by ascending coverage percentage. Entries can be
filtered by a path prefix so you can focus on application code (e.g. `src/`).

Example usage (run from repo root):

    uv run python scripts/coverage_report.py
    uv run python scripts/coverage_report.py --prefix src/cdt --limit 5
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Optional

DEFAULT_REPORT = Path("tarpaulin-report.json")


@dataclass(frozen=True)
class CoverageEntry:
    coverage: float
    coverable: int
    covered: int
    path: Path

    def format(self, relative_to: Optional[Path] = None) -> str:
        display_path = self._relative_path(relative_to)
        return f"{self.coverage:6.2f}%  {display_path}"

    def _relative_path(self, relative_to: Optional[Path]) -> Path:
        if relative_to is None:
            return self.path
        try:
            return self.path.relative_to(relative_to)
        except ValueError:
            return self.path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Summarize Tarpaulin JSON coverage report."
    )
    parser.add_argument(
        "--report",
        type=Path,
        default=DEFAULT_REPORT,
        help="Path to tarpaulin JSON report (default: %(default)s).",
    )
    parser.add_argument(
        "--prefix",
        default="",
        help=(
            "Only include files whose (relative) path starts with this prefix. "
            "Use empty string to include all."
        ),
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit output to the N lowest-covered entries.",
    )
    parser.add_argument(
        "--descending",
        action="store_true",
        help="Sort in descending order (default: ascending).",
    )
    return parser.parse_args()


def load_report(report_path: Path) -> dict:
    if not report_path.is_file():
        raise SystemExit(f"Coverage report not found: {report_path}")
    with report_path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def coverage_entries(data: dict) -> Iterable[CoverageEntry]:
    files = data.get("files", [])
    for entry in files:
        coverable = entry.get("coverable", 0)
        covered = entry.get("covered", 0)
        if not coverable:
            continue
        raw_path = entry.get("path")
        if not raw_path:
            continue
        if isinstance(raw_path, (list, tuple)):
            path = Path(*raw_path)
        else:
            path = Path(raw_path)
        coverage = (covered / coverable) * 100
        yield CoverageEntry(coverage=coverage, coverable=coverable, covered=covered, path=path)


def filter_entries(
    entries: Iterable[CoverageEntry],
    prefix: str,
    relative_to: Path,
) -> List[CoverageEntry]:
    if not prefix:
        return list(entries)
    normalized_prefix = prefix if prefix.endswith("/") else f"{prefix}/"
    filtered: List[CoverageEntry] = []
    for entry in entries:
        relative = entry._relative_path(relative_to)
        relative_str = relative.as_posix()
        if relative_str.startswith(normalized_prefix):
            filtered.append(entry)
    return filtered


def main() -> None:
    args = parse_args()
    data = load_report(args.report)

    repo_root = Path(__file__).resolve().parent.parent
    entries = list(coverage_entries(data))
    filtered = filter_entries(entries, args.prefix, repo_root)

    if not filtered:
        prefix_message = f" with prefix '{args.prefix}'" if args.prefix else ""
        print(f"No coverable files found{prefix_message}.")
        return

    sorted_entries = sorted(filtered, key=lambda item: item.coverage, reverse=args.descending)
    if args.limit is not None:
        sorted_entries = sorted_entries[: args.limit]

    for entry in sorted_entries:
        print(entry.format(relative_to=repo_root))


if __name__ == "__main__":
    main()