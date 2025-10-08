"""
Advanced performance analysis and reporting for CDT benchmarks.
Provides detailed statistics, trend analysis, and regression detection.
"""

import json
import sys
import argparse
import math
import statistics
import shutil
from pathlib import Path
from datetime import datetime, timedelta
from typing import Dict, Optional
import subprocess


class PerformanceAnalyzer:
    """Advanced performance analysis for CDT benchmarks."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.baseline_dir = project_root / "performance_baselines"
        self.results_dir = project_root / "target" / "criterion"
        self.reports_dir = project_root / "performance_reports"

        # Ensure directories exist
        self.baseline_dir.mkdir(exist_ok=True)
        self.reports_dir.mkdir(exist_ok=True)

    def run_benchmarks(self) -> bool:
        """Run cargo bench and return success status."""
        print("🏃 Running benchmarks...")
        try:
            result = subprocess.run(
                ["cargo", "bench", "--message-format=json"],
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=600,  # 10 minute timeout
            )

            if result.returncode != 0:
                print("❌ Benchmark execution failed:")
                print(result.stderr)
                return False

            print("✅ Benchmarks completed successfully")
            return True

        except subprocess.TimeoutExpired:
            print("❌ Benchmark execution timed out")
            return False
        except Exception as e:
            print(f"❌ Error running benchmarks: {e}")
            return False

    def extract_criterion_results(self) -> Dict:
        """Extract benchmark results from criterion output directory."""
        results = {}

        if not self.results_dir.exists():
            print(f"Warning: Criterion results directory not found: {self.results_dir}")
            return results

        # Recursively find all estimates.json files (Criterion writes current runs to "new")
        for estimates_file in self.results_dir.rglob("estimates.json"):
            if estimates_file.parent.name not in {"new", "base"}:
                continue
            try:
                with open(estimates_file) as f:
                    data = json.load(f)

                # Build benchmark name from path structure
                # e.g., action_calculations/calculate_action/50/base/estimates.json
                # becomes "action_calculations/calculate_action/50"
                path_parts = estimates_file.relative_to(self.results_dir).parts[
                    :-2
                ]  # Remove '<run_type>/estimates.json'
                benchmark_name = "/".join(path_parts)

                results[benchmark_name] = {
                    "mean_ns": data.get("mean", {}).get("point_estimate", 0),
                    "std_dev_ns": data.get("std_dev", {}).get("point_estimate", 0),
                    "median_ns": data.get("median", {}).get("point_estimate", 0),
                    "mad_ns": data.get("median_abs_dev", {}).get("point_estimate", 0),
                    "timestamp": datetime.now().isoformat(),
                }

                # Add confidence intervals if available
                mean_ci = data.get("mean", {}).get("confidence_interval", {})
                if mean_ci:
                    results[benchmark_name]["mean_ci_lower"] = mean_ci.get(
                        "lower_bound", 0
                    )
                    results[benchmark_name]["mean_ci_upper"] = mean_ci.get(
                        "upper_bound", 0
                    )

            except (json.JSONDecodeError, KeyError) as e:
                print(f"Warning: Could not parse {estimates_file}: {e}")

        return results

    def save_baseline(self, results: Dict, tag: Optional[str] = None) -> Path:
        """Save current results as a baseline."""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        if tag:
            filename = f"baseline_{tag}_{timestamp}.json"
        else:
            filename = f"baseline_{timestamp}.json"

        baseline_file = self.baseline_dir / filename
        with open(baseline_file, "w") as f:
            json.dump(results, f, indent=2)

        # Update latest symlink (with Windows fallback)
        # On Windows, symlinks require elevated privileges, so we fall back to file copying
        latest_link = self.baseline_dir / "latest.json"
        try:
            latest_link.unlink(missing_ok=True)
            latest_link.symlink_to(filename)
        except (OSError, NotImplementedError):
            # Fallback for Windows or systems without symlink support
            # Use file copy instead of symlink to ensure cross-platform compatibility
            if latest_link.exists():
                latest_link.unlink()
            shutil.copy2(baseline_file, latest_link)

        print(f"✅ Saved baseline: {baseline_file}")
        return baseline_file

    def load_baseline(self, baseline_path: Optional[Path] = None) -> Dict:
        """Load baseline results."""
        if baseline_path is None:
            baseline_path = self.baseline_dir / "latest.json"

        if not baseline_path.exists():
            return {}

        try:
            with open(baseline_path) as f:
                return json.load(f)
        except (json.JSONDecodeError, FileNotFoundError) as e:
            print(f"Warning: Could not load baseline {baseline_path}: {e}")
            return {}

    def compare_results(
        self, current: Dict, baseline: Dict, threshold: float = 10.0
    ) -> Dict:
        """Compare current results with baseline and categorize changes."""
        comparison = {
            "regressions": [],
            "improvements": [],
            "new_benchmarks": [],
            "stable": [],
            "summary": {},
        }

        for benchmark, current_data in current.items():
            if benchmark not in baseline:
                comparison["new_benchmarks"].append(
                    {"benchmark": benchmark, "mean_ns": current_data["mean_ns"]}
                )
                continue

            current_mean = current_data["mean_ns"]
            baseline_mean = baseline[benchmark]["mean_ns"]

            if baseline_mean == 0:
                continue

            change_percent = ((current_mean - baseline_mean) / baseline_mean) * 100

            change_data = {
                "benchmark": benchmark,
                "change_percent": change_percent,
                "current_ns": current_mean,
                "baseline_ns": baseline_mean,
                "current_std": current_data.get("std_dev_ns", 0),
                "baseline_std": baseline.get(benchmark, {}).get("std_dev_ns", 0),
            }

            if change_percent > threshold:
                comparison["regressions"].append(change_data)
            elif change_percent < -threshold:
                comparison["improvements"].append(change_data)
            else:
                comparison["stable"].append(change_data)

        # Calculate summary statistics
        all_changes = [
            item["change_percent"]
            for item in (
                comparison["regressions"]
                + comparison["improvements"]
                + comparison["stable"]
            )
        ]

        if all_changes:
            comparison["summary"] = {
                "total_benchmarks": len(current),
                "regressions": len(comparison["regressions"]),
                "improvements": len(comparison["improvements"]),
                "stable": len(comparison["stable"]),
                "new": len(comparison["new_benchmarks"]),
                "avg_change": statistics.mean(all_changes),
                "median_change": statistics.median(all_changes),
                "max_regression": max(
                    [r["change_percent"] for r in comparison["regressions"]], default=0
                ),
                "max_improvement": max(
                    (abs(i["change_percent"]) for i in comparison["improvements"]),
                    default=0,
                ),
            }

        return comparison

    def format_time_ns(self, nanoseconds: float) -> str:
        """Format nanoseconds into human-readable time units."""
        if nanoseconds < 1000:
            return f"{nanoseconds:.1f}ns"
        elif nanoseconds < 1_000_000:
            return f"{nanoseconds / 1000:.1f}µs"
        elif nanoseconds < 1_000_000_000:
            return f"{nanoseconds / 1_000_000:.1f}ms"
        else:
            return f"{nanoseconds / 1_000_000_000:.2f}s"

    def print_comparison_results(self, comparison: Dict):
        """Print comparison results to console with colors."""
        summary = comparison.get("summary", {})

        if comparison["regressions"]:
            print("🔴 PERFORMANCE REGRESSIONS DETECTED:")
            for reg in sorted(
                comparison["regressions"],
                key=lambda x: x["change_percent"],
                reverse=True,
            ):
                current_time = self.format_time_ns(reg["current_ns"])
                baseline_time = self.format_time_ns(reg["baseline_ns"])
                print(f"  {reg['benchmark']}: +{reg['change_percent']:.1f}% slower")
                print(f"    Current: {current_time}, Baseline: {baseline_time}")
            print()

        if comparison["improvements"]:
            print("🟢 PERFORMANCE IMPROVEMENTS:")
            for imp in sorted(
                comparison["improvements"],
                key=lambda x: abs(x["change_percent"]),
                reverse=True,
            ):
                current_time = self.format_time_ns(imp["current_ns"])
                baseline_time = self.format_time_ns(imp["baseline_ns"])
                improvement_pct = abs(imp["change_percent"])
                print(f"  {imp['benchmark']}: +{improvement_pct:.1f}% faster")
                print(f"    Current: {current_time}, Baseline: {baseline_time}")
            print()

        if comparison["new_benchmarks"]:
            print("🆕 NEW BENCHMARKS:")
            for bench in comparison["new_benchmarks"]:
                time_str = self.format_time_ns(bench["mean_ns"])
                print(f"  {bench['benchmark']}: {time_str}")
            print()

        if summary:
            print("📈 SUMMARY:")
            print(f"  Total benchmarks: {summary.get('total_benchmarks', 0)}")
            print(f"  Regressions: {summary.get('regressions', 0)}")
            print(f"  Improvements: {summary.get('improvements', 0)}")
            print(f"  Stable: {summary.get('stable', 0)}")
            print(f"  New: {summary.get('new', 0)}")
            avg_change = summary.get("avg_change")
            median_change = summary.get("median_change")
            if avg_change is not None and median_change is not None:
                print(f"  Average change: {avg_change:.1f}%")
                print(f"  Median change: {median_change:.1f}%")
            if summary.get("max_regression"):
                print(f"  Max regression: +{summary['max_regression']:.1f}%")
            if summary.get("max_improvement"):
                print(f"  Max improvement: +{summary['max_improvement']:.1f}%")
            print()

        if (
            not comparison["regressions"]
            and not comparison["improvements"]
            and not comparison["new_benchmarks"]
        ):
            print("✅ No significant performance changes detected")

    def generate_report(
        self, comparison: Dict, output_file: Optional[Path] = None
    ) -> str:
        """Generate a detailed performance report."""
        lines = []
        lines.append("# CDT Performance Analysis Report")
        lines.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        lines.append("")

        summary = comparison.get("summary", {})
        if summary:
            lines.append("## Summary")
            lines.append(f"- Total benchmarks: {summary['total_benchmarks']}")
            lines.append(f"- Regressions: {summary['regressions']}")
            lines.append(f"- Improvements: {summary['improvements']}")
            lines.append(f"- Stable: {summary['stable']}")
            lines.append(f"- New benchmarks: {summary['new']}")
            lines.append(f"- Average change: {summary['avg_change']:.1f}%")
            lines.append(f"- Median change: {summary['median_change']:.1f}%")
            lines.append("")

        if comparison["regressions"]:
            lines.append("## 🔴 Performance Regressions")
            lines.append("| Benchmark | Change | Current | Baseline | Ratio |")
            lines.append("|-----------|--------|---------|----------|-------|")

            for reg in sorted(
                comparison["regressions"],
                key=lambda x: x["change_percent"],
                reverse=True,
            ):
                current_time = self.format_time_ns(reg["current_ns"])
                baseline_time = self.format_time_ns(reg["baseline_ns"])
                ratio = (
                    reg["current_ns"] / reg["baseline_ns"]
                    if reg["baseline_ns"] != 0
                    else float("inf")
                )
                ratio_display = "∞" if math.isinf(ratio) else f"{ratio:.2f}"

                lines.append(
                    f"| {reg['benchmark']} | +{reg['change_percent']:.1f}% | {current_time} | {baseline_time} | {ratio_display}x |"
                )
            lines.append("")

        if comparison["improvements"]:
            lines.append("## 🟢 Performance Improvements")
            lines.append("| Benchmark | Change | Current | Baseline | Ratio |")
            lines.append("|-----------|--------|---------|----------|-------|")

            for imp in sorted(
                comparison["improvements"],
                key=lambda x: abs(x["change_percent"]),
                reverse=True,
            ):
                current_time = self.format_time_ns(imp["current_ns"])
                baseline_time = self.format_time_ns(imp["baseline_ns"])
                ratio = (
                    imp["baseline_ns"] / imp["current_ns"]
                    if imp["current_ns"] != 0
                    else float("inf")
                )
                ratio_display = "∞" if math.isinf(ratio) else f"{ratio:.2f}"
                improvement_pct = abs(imp["change_percent"])

                lines.append(
                    f"| {imp['benchmark']} | -{improvement_pct:.1f}% | {current_time} | {baseline_time} | {ratio_display}x |"
                )
            lines.append("")

        if comparison["new_benchmarks"]:
            lines.append("## 🆕 New Benchmarks")
            for bench in comparison["new_benchmarks"]:
                time_str = self.format_time_ns(bench["mean_ns"])
                lines.append(f"- {bench['benchmark']}: {time_str}")
            lines.append("")

        if comparison["stable"]:
            lines.append("## ✅ Stable Benchmarks")
            lines.append(
                f"No significant changes detected in {len(comparison['stable'])} benchmarks."
            )
            lines.append("")

        report_content = "\n".join(lines)

        if output_file:
            with open(output_file, "w") as f:
                f.write(report_content)
            print(f"📄 Report saved to: {output_file}")

        return report_content

    def analyze_trends(self, days: int = 30) -> Dict:
        """Analyze performance trends over the specified number of days."""
        cutoff_date = datetime.now() - timedelta(days=days)
        baselines = []

        for baseline_file in self.baseline_dir.glob("baseline_*.json"):
            # Extract timestamp from filename
            try:
                timestamp_str = (
                    baseline_file.stem.split("_")[-2]
                    + "_"
                    + baseline_file.stem.split("_")[-1]
                )
                timestamp = datetime.strptime(timestamp_str, "%Y%m%d_%H%M%S")

                if timestamp >= cutoff_date:
                    with open(baseline_file) as f:
                        data = json.load(f)
                        baselines.append(
                            {
                                "timestamp": timestamp,
                                "file": baseline_file,
                                "data": data,
                            }
                        )
            except (ValueError, json.JSONDecodeError, IndexError):
                continue

        if len(baselines) < 2:
            return {"error": "Not enough historical data for trend analysis"}

        baselines.sort(key=lambda x: x["timestamp"])

        # Analyze trends for each benchmark
        trends = {}
        benchmark_names = set()
        for baseline in baselines:
            benchmark_names.update(baseline["data"].keys())

        for benchmark in benchmark_names:
            values = []
            timestamps = []

            for baseline in baselines:
                if benchmark in baseline["data"]:
                    values.append(baseline["data"][benchmark]["mean_ns"])
                    timestamps.append(baseline["timestamp"])

            if len(values) >= 2:
                # Calculate trend (simple linear regression slope)
                n = len(values)
                sum_x = sum(range(n))
                sum_y = sum(values)
                sum_xy = sum(i * v for i, v in enumerate(values))
                sum_xx = sum(i * i for i in range(n))

                denominator = n * sum_xx - sum_x * sum_x
                if denominator == 0:
                    # All data points at same position - treat as stable
                    slope = 0
                else:
                    slope = (n * sum_xy - sum_x * sum_y) / denominator

                # Use small epsilon for floating point comparison
                epsilon = 1e-9
                trends[benchmark] = {
                    "slope": slope,
                    "trend": "improving"
                    if slope < 0
                    else "degrading"
                    if slope > 0
                    else "stable",
                    "data_points": n,
                    "first_value": values[0],
                    "last_value": values[-1],
                    "change_percent": (
                        ((values[-1] - values[0]) / values[0]) * 100
                        if abs(values[0]) > epsilon
                        else 0
                    ),
                }

        return {
            "period_days": days,
            "baselines_analyzed": len(baselines),
            "trends": trends,
        }


def main():
    parser = argparse.ArgumentParser(
        description="CDT Performance Analysis Tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run benchmarks and compare with baseline (equivalent to check_performance.sh)
  ./performance_analysis.py
  
  # Save current results as baseline
  ./performance_analysis.py --save-baseline
  
  # Save baseline with tag
  ./performance_analysis.py --save-baseline --tag "v1.0.0"
  
  # Compare with custom threshold
  ./performance_analysis.py --threshold 5.0
  
  # Generate detailed report
  ./performance_analysis.py --report performance_report.md
  
  # Analyze trends over last 7 days
  ./performance_analysis.py --trends 7
        """,
    )
    parser.add_argument(
        "--save-baseline", action="store_true", help="Save current results as baseline"
    )
    parser.add_argument("--tag", help="Tag for saved baseline")
    parser.add_argument(
        "--threshold",
        type=float,
        default=10.0,
        help="Regression threshold percentage (default: 10.0)",
    )
    parser.add_argument("--compare", help="Compare with specific baseline file")
    parser.add_argument(
        "--project-root", type=Path, help="Path to the project root directory"
    )
    parser.add_argument("--report", help="Generate detailed report to specified file")
    parser.add_argument(
        "--trends", type=int, metavar="DAYS", help="Analyze trends over N days"
    )
    parser.add_argument(
        "--no-run",
        action="store_true",
        help="Skip running benchmarks, use existing results",
    )
    parser.add_argument("--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    project_root: Optional[Path] = None

    if args.project_root:
        provided_root = args.project_root.resolve()
        if not (
            (provided_root / "Cargo.toml").exists() or (provided_root / ".git").exists()
        ):
            print("❌ Provided project root does not contain Cargo.toml or .git")
            return 1
        project_root = provided_root
    else:
        current = Path(__file__).resolve().parent
        while current != current.parent:
            if (current / "Cargo.toml").exists() or (current / ".git").exists():
                project_root = current
                break
            current = current.parent

    if project_root is None:
        print("❌ Could not detect project root (no Cargo.toml or .git found)")
        print(
            "   Please run this script from within the project or specify --project-root"
        )
        return 1

    analyzer = PerformanceAnalyzer(project_root)

    # Handle trend analysis
    if args.trends:
        print(f"📊 Analyzing performance trends over {args.trends} days...")
        trends = analyzer.analyze_trends(args.trends)

        if "error" in trends:
            print(f"❌ {trends['error']}")
            return 1

        print(
            f"Analyzed {trends['baselines_analyzed']} baselines over {trends['period_days']} days"
        )

        degrading = [
            name
            for name, trend in trends["trends"].items()
            if trend["trend"] == "degrading"
        ]
        improving = [
            name
            for name, trend in trends["trends"].items()
            if trend["trend"] == "improving"
        ]

        if degrading:
            print(f"\n🔴 Degrading trends ({len(degrading)} benchmarks):")
            for bench in degrading:
                change = trends["trends"][bench]["change_percent"]
                print(f"  {bench}: {change:+.1f}% over period")

        if improving:
            print(f"\n🟢 Improving trends ({len(improving)} benchmarks):")
            for bench in improving:
                change = trends["trends"][bench]["change_percent"]
                print(f"  {bench}: {change:+.1f}% over period")

        return 0

    # Run benchmarks unless --no-run specified
    if not args.no_run:
        if not analyzer.run_benchmarks():
            return 1

    # Extract current results
    print("🔍 Extracting benchmark results...")
    current_results = analyzer.extract_criterion_results()

    if not current_results:
        print(
            "❌ No benchmark results found. Run 'cargo bench' first or remove --no-run flag."
        )
        return 1

    print(f"Found {len(current_results)} benchmark results")

    # Handle saving baseline
    if args.save_baseline:
        analyzer.save_baseline(current_results, args.tag)
        return 0

    # Load baseline for comparison
    baseline_file = Path(args.compare) if args.compare else None
    baseline = analyzer.load_baseline(baseline_file)

    if not baseline:
        print("⚠️  No baseline found for comparison.")
        print("   Run with --save-baseline to create an initial baseline.")
        return 0

    # Compare results
    print("📊 Comparing with baseline...")
    comparison = analyzer.compare_results(current_results, baseline, args.threshold)

    # Print results to console
    analyzer.print_comparison_results(comparison)

    # Generate detailed report if requested
    if args.report:
        report_file = Path(args.report)
        analyzer.generate_report(comparison, report_file)

    # Print summary
    summary = comparison.get("summary", {})
    if summary:
        print("\n📈 Performance Summary:")
        print(f"   Total benchmarks: {summary['total_benchmarks']}")
        print(f"   Regressions: {summary['regressions']}")
        print(f"   Improvements: {summary['improvements']}")
        print(f"   Stable: {summary['stable']}")
        print(f"   New: {summary['new']}")

        if summary["regressions"] > 0:
            print(f"   Max regression: +{summary['max_regression']:.1f}%")

        if summary["improvements"] > 0:
            print(f"   Max improvement: +{summary['max_improvement']:.1f}%")

    # Exit with error code if regressions found (for CI)
    return 1 if comparison["regressions"] else 0


if __name__ == "__main__":
    sys.exit(main())
