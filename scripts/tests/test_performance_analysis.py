"""Tests for scripts/performance_analysis.py.

These tests focus on pure, filesystem-only behavior (no cargo benchmarks).
"""

from __future__ import annotations

import json
import sys
from datetime import UTC, datetime, timedelta
from pathlib import Path

import pytest

# Add scripts directory to path so we can import modules as top-level scripts/*.
sys.path.insert(0, str(Path(__file__).parent.parent))

from typing import TYPE_CHECKING, cast

from performance_analysis import PerformanceAnalyzer

if TYPE_CHECKING:
    from performance_analysis import TrendAnalysisSuccess


def _write_baseline(
    project_root: Path,
    filename: str,
    *,
    means_by_benchmark: dict[str, float],
) -> None:
    baseline_dir = project_root / "performance_baselines"
    baseline_dir.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.now(UTC).isoformat(timespec="seconds")
    payload = {
        name: {
            "mean_ns": mean_ns,
            "std_dev_ns": 0.0,
            "median_ns": mean_ns,
            "mad_ns": 0.0,
            "timestamp": timestamp,
        }
        for name, mean_ns in means_by_benchmark.items()
    }

    (baseline_dir / filename).write_text(json.dumps(payload), encoding="utf-8")


def _baseline_filename(tag: str, ts: datetime) -> str:
    timestamp_str = ts.strftime("%Y%m%d_%H%M%S")
    return f"baseline_{tag}_{timestamp_str}.json" if tag else f"baseline_{timestamp_str}.json"


def test_analyze_trends_errors_with_fewer_than_two_baselines(tmp_path: Path) -> None:
    analyzer = PerformanceAnalyzer(tmp_path)

    now = datetime.now(UTC)
    _write_baseline(
        tmp_path,
        _baseline_filename("", now),
        means_by_benchmark={"bench": 100.0},
    )

    result = analyzer.analyze_trends(days=30)
    assert "error" in result


def test_analyze_trends_classifies_improving_stable_degrading(tmp_path: Path) -> None:
    analyzer = PerformanceAnalyzer(tmp_path)

    now = datetime.now(UTC)
    timestamps = [now - timedelta(days=2), now - timedelta(days=1), now]

    # decreasing -> improving, increasing -> degrading, constant -> stable
    series = [
        {"improving": 120.0, "degrading": 100.0, "stable": 50.0},
        {"improving": 110.0, "degrading": 110.0, "stable": 50.0},
        {"improving": 100.0, "degrading": 120.0, "stable": 50.0},
    ]

    for ts, means in zip(timestamps, series, strict=True):
        _write_baseline(tmp_path, _baseline_filename("", ts), means_by_benchmark=means)

    result = analyzer.analyze_trends(days=30)
    assert "error" not in result

    success = cast("TrendAnalysisSuccess", result)
    trends = success["trends"]
    assert trends["improving"]["trend"] == "improving"
    assert trends["degrading"]["trend"] == "degrading"
    assert trends["stable"]["trend"] == "stable"

    assert trends["degrading"]["change_percent"] == pytest.approx(20.0)
    assert trends["improving"]["change_percent"] == pytest.approx(((100.0 - 120.0) / 120.0) * 100.0)
    assert trends["stable"]["change_percent"] == pytest.approx(0.0)


def test_analyze_trends_parses_tagged_filenames_and_sorts_by_timestamp(tmp_path: Path) -> None:
    analyzer = PerformanceAnalyzer(tmp_path)

    now = datetime.now(UTC)
    earlier = now - timedelta(days=2)

    # Tag contains underscores; timestamp must still be extracted from the end.
    _write_baseline(
        tmp_path,
        _baseline_filename("feature_branch_with_underscores", earlier),
        means_by_benchmark={"bench": 100.0},
    )
    _write_baseline(
        tmp_path,
        _baseline_filename("ci", now),
        means_by_benchmark={"bench": 200.0},
    )

    result = analyzer.analyze_trends(days=30)
    assert "error" not in result

    success = cast("TrendAnalysisSuccess", result)
    trend = success["trends"]["bench"]
    assert trend["first_value"] == pytest.approx(100.0)
    assert trend["last_value"] == pytest.approx(200.0)
    assert trend["trend"] == "degrading"
    assert trend["change_percent"] == pytest.approx(100.0)


def test_analyze_trends_ignores_invalid_json_baselines(tmp_path: Path) -> None:
    analyzer = PerformanceAnalyzer(tmp_path)

    now = datetime.now(UTC)
    earlier = now - timedelta(days=2)
    middle = now - timedelta(days=1)

    _write_baseline(
        tmp_path,
        _baseline_filename("", earlier),
        means_by_benchmark={"bench": 100.0},
    )

    # This file matches the baseline filename pattern, but contains invalid JSON and should be skipped.
    baseline_dir = tmp_path / "performance_baselines"
    baseline_dir.mkdir(parents=True, exist_ok=True)
    (baseline_dir / _baseline_filename("invalid_json", middle)).write_text("{not valid json", encoding="utf-8")

    _write_baseline(
        tmp_path,
        _baseline_filename("", now),
        means_by_benchmark={"bench": 200.0},
    )

    result = analyzer.analyze_trends(days=30)
    assert "error" not in result

    success = cast("TrendAnalysisSuccess", result)
    trend = success["trends"]["bench"]
    assert trend["first_value"] == pytest.approx(100.0)
    assert trend["last_value"] == pytest.approx(200.0)
    assert trend["trend"] == "degrading"
    assert trend["change_percent"] == pytest.approx(100.0)


def test_analyze_trends_handles_benchmarks_missing_in_some_baselines(tmp_path: Path) -> None:
    analyzer = PerformanceAnalyzer(tmp_path)

    now = datetime.now(UTC)
    earlier = now - timedelta(days=2)
    middle = now - timedelta(days=1)

    # "always" appears in every baseline -> stable
    # "sometimes" appears in two baselines -> trend computed from available data
    # "once" appears in one baseline -> excluded from results (needs >= 2 data points)
    _write_baseline(
        tmp_path,
        _baseline_filename("", earlier),
        means_by_benchmark={"always": 10.0, "sometimes": 100.0, "once": 5.0},
    )
    _write_baseline(
        tmp_path,
        _baseline_filename("", middle),
        means_by_benchmark={"always": 10.0},
    )
    _write_baseline(
        tmp_path,
        _baseline_filename("", now),
        means_by_benchmark={"always": 10.0, "sometimes": 50.0},
    )

    result = analyzer.analyze_trends(days=30)
    assert "error" not in result

    success = cast("TrendAnalysisSuccess", result)
    trends = success["trends"]

    assert trends["always"]["trend"] == "stable"
    assert trends["always"]["change_percent"] == pytest.approx(0.0)

    assert trends["sometimes"]["trend"] == "improving"
    assert trends["sometimes"]["first_value"] == pytest.approx(100.0)
    assert trends["sometimes"]["last_value"] == pytest.approx(50.0)
    assert trends["sometimes"]["change_percent"] == pytest.approx(-50.0)

    assert "once" not in trends
