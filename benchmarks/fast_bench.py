"""Benchmark conversions from TextGrid files to tabular and structured data.

This script compares three implementations:
1. `pytextgrid` (this project)
2. The pure Python `textgrid` package
3. `parselmouth`

Two scenarios are evaluated:
- Many small TextGrid files (generated on the fly)
- One very large TextGrid file

For each scenario we measure conversions to:
- A Pandas DataFrame
- A structured tuple that mirrors the `pytextgrid` data representation

Results are aggregated, displayed, and visualised as a bar chart.
"""

from __future__ import annotations

import argparse
import time
from dataclasses import dataclass
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Callable, Iterable, List, Sequence

import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import parselmouth
from textgrid import IntervalTier as TGIntervalTier
from textgrid import TextGrid as TGTextGrid

from pytextgrid import data_to_textgrid, textgrid_to_data, textgrid_to_df


@dataclass
class Scenario:
    name: str
    description: str
    files: List[Path]


@dataclass
class BenchmarkMethod:
    name: str
    to_dataframe: Callable[[Sequence[Path]], pd.DataFrame]
    to_structured: Callable[[Sequence[Path]], object]


def _ensure_files(files: Sequence[Path]) -> List[Path]:
    return [Path(f) for f in files]


def _generate_tier_items(
    count: int, step: float, offset: float, prefix: str, *, is_interval: bool
) -> list[tuple[float, float, str]]:
    # The generator keeps timestamps deterministic so benchmarking stays reproducible.
    items: list[tuple[float, float, str]] = []
    for idx in range(count):
        start = offset + idx * step
        if is_interval:
            items.append((start, start + step, f"{prefix}_{idx}"))
        else:
            items.append((start, start, f"{prefix}_{idx}"))
    return items


def _build_textgrid_payload(
    *,
    n_interval_tiers: int,
    n_point_tiers: int,
    interval_items: int,
    point_items: int,
    step: float,
) -> tuple[float, float, list[tuple[str, bool, list[tuple[float, float, str]]]]]:
    tiers: list[tuple[str, bool, list[tuple[float, float, str]]]] = []
    total_items = max(interval_items, point_items)
    tmax = total_items * step if total_items else 0.0
    for tid in range(n_interval_tiers):
        tiers.append(
            (
                f"interval_{tid}",
                True,
                _generate_tier_items(
                    interval_items, step, 0.0, f"i{tid}", is_interval=True
                ),
            )
        )
    for tid in range(n_point_tiers):
        tiers.append(
            (
                f"point_{tid}",
                False,
                _generate_tier_items(
                    point_items, step, step / 2.0, f"p{tid}", is_interval=False
                ),
            )
        )
    return (0.0, tmax, tiers)


def _write_textgrid(
    path: Path,
    payload: tuple[
        float, float, list[tuple[str, bool, list[tuple[float, float, str]]]]
    ],
    file_type: str = "long",
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    data_to_textgrid(payload, str(path), file_type=file_type)


def create_many_small_files(
    base_dir: Path,
    *,
    file_count: int,
    interval_tiers: int,
    point_tiers: int,
    interval_items: int,
    point_items: int,
    step: float,
) -> Scenario:
    files: list[Path] = []
    payload = _build_textgrid_payload(
        n_interval_tiers=interval_tiers,
        n_point_tiers=point_tiers,
        interval_items=interval_items,
        point_items=point_items,
        step=step,
    )
    for idx in range(file_count):
        file_path = base_dir / "many_small" / f"sample_{idx:04d}.TextGrid"
        _write_textgrid(file_path, payload)
        files.append(file_path)
    total_tiers = interval_tiers + point_tiers
    description = (
        f"{file_count} files with {total_tiers} tiers; "
        f"interval items={interval_items}, point items={point_items}"
    )
    return Scenario(name="many-small-files", description=description, files=files)


def create_single_large_file(
    base_dir: Path,
    *,
    interval_tiers: int,
    point_tiers: int,
    interval_items: int,
    point_items: int,
    step: float,
) -> Scenario:
    payload = _build_textgrid_payload(
        n_interval_tiers=interval_tiers,
        n_point_tiers=point_tiers,
        interval_items=interval_items,
        point_items=point_items,
        step=step,
    )
    file_path = base_dir / "single_large" / "large.TextGrid"
    _write_textgrid(file_path, payload)
    total_tiers = interval_tiers + point_tiers
    description = (
        f"Single file with {total_tiers} tiers; "
        f"interval items={interval_items}, point items={point_items}"
    )
    return Scenario(
        name="single-large-file", description=description, files=[file_path]
    )


def pytextgrid_to_dataframe(files: Sequence[Path]) -> pd.DataFrame:
    return textgrid_to_df(
        [Path(f) for f in files], file_name_column=True, backend="pandas"
    )


def pytextgrid_to_structured(files: Sequence[Path]) -> object:
    if len(files) == 1:
        return textgrid_to_data(files[0])
    return textgrid_to_data([Path(f) for f in files])


def textgridpkg_to_dataframe(files: Sequence[Path]) -> pd.DataFrame:
    rows: list[dict[str, object]] = []
    for fpath in files:
        tg = TGTextGrid.fromFile(str(fpath))
        filename = str(fpath)
        for tier in tg.tiers:
            is_interval = isinstance(tier, TGIntervalTier)
            tier_name = tier.name or ""
            for item in tier:
                if is_interval:
                    tmin = float(item.minTime)
                    tmax = float(item.maxTime)
                    label = item.mark
                else:
                    tmin = tmax = float(item.time)
                    label = item.mark
                rows.append(
                    {
                        "tmin": tmin,
                        "tmax": tmax,
                        "label": label,
                        "tier": tier_name,
                        "is_interval": is_interval,
                        "filename": filename,
                    }
                )
    return pd.DataFrame(rows)


def textgridpkg_to_structured(files: Sequence[Path]) -> object:
    result: dict[
        str, tuple[float, float, list[tuple[str, bool, list[tuple[float, float, str]]]]]
    ] = {}
    for fpath in files:
        tg = TGTextGrid.fromFile(str(fpath))
        tiers: list[tuple[str, bool, list[tuple[float, float, str]]]] = []
        for tier in tg.tiers:
            is_interval = isinstance(tier, TGIntervalTier)
            tier_name = tier.name or ""
            items: list[tuple[float, float, str]] = []
            for item in tier:
                if is_interval:
                    items.append((float(item.minTime), float(item.maxTime), item.mark))
                else:
                    time = float(item.time)
                    items.append((time, time, item.mark))
            tiers.append((tier_name, is_interval, items))
        result[str(fpath)] = (float(tg.minTime), float(tg.maxTime), tiers)
    if len(files) > 1:
        return result
    return next(iter(result.values()))


def parselmouth_to_dataframe(files: Sequence[Path]) -> pd.DataFrame:
    rows: list[dict[str, object]] = []
    for fpath in files:
        tgt = parselmouth.TextGrid.read(str(fpath))
        filename = str(fpath)
        ntier = parselmouth.praat.call(tgt, "Get number of tiers")
        for i in range(1, ntier + 1):
            tier_name = parselmouth.praat.call(tgt, "Get tier name", i)
            is_interval = bool(parselmouth.praat.call(tgt, "Is interval tier", i))
            if is_interval:
                nitems = parselmouth.praat.call(tgt, "Get number of intervals", i)
                for j in range(1, nitems + 1):
                    tmin = float(
                        parselmouth.praat.call(tgt, "Get start time of interval", i, j)
                    )
                    tmax = float(
                        parselmouth.praat.call(tgt, "Get end time of interval", i, j)
                    )
                    text = parselmouth.praat.call(tgt, "Get label of interval", i, j)
                    rows.append(
                        {
                            "tmin": tmin,
                            "tmax": tmax,
                            "label": text,
                            "tier": tier_name,
                            "is_interval": True,
                            "filename": filename,
                        }
                    )
            else:
                nitems = parselmouth.praat.call(tgt, "Get number of points", i)
                for j in range(1, nitems + 1):
                    time_val = float(
                        parselmouth.praat.call(tgt, "Get time of point", i, j)
                    )
                    text = parselmouth.praat.call(tgt, "Get label of point", i, j)
                    rows.append(
                        {
                            "tmin": time_val,
                            "tmax": time_val,
                            "label": text,
                            "tier": tier_name,
                            "is_interval": False,
                            "filename": filename,
                        }
                    )
    return pd.DataFrame(rows)


def parselmouth_to_structured(files: Sequence[Path]) -> object:
    result: dict[
        str, tuple[float, float, list[tuple[str, bool, list[tuple[float, float, str]]]]]
    ] = {}
    for fpath in files:
        tgt = parselmouth.TextGrid.read(str(fpath))
        tmin = float(parselmouth.praat.call(tgt, "Get start time"))
        tmax = float(parselmouth.praat.call(tgt, "Get end time"))
        tiers: list[tuple[str, bool, list[tuple[float, float, str]]]] = []
        ntier = parselmouth.praat.call(tgt, "Get number of tiers")
        for i in range(1, ntier + 1):
            tier_name = parselmouth.praat.call(tgt, "Get tier name", i)
            is_interval = bool(parselmouth.praat.call(tgt, "Is interval tier", i))
            items: list[tuple[float, float, str]] = []
            if is_interval:
                nitems = parselmouth.praat.call(tgt, "Get number of intervals", i)
                for j in range(1, nitems + 1):
                    start = float(
                        parselmouth.praat.call(tgt, "Get start time of interval", i, j)
                    )
                    end = float(
                        parselmouth.praat.call(tgt, "Get end time of interval", i, j)
                    )
                    label = parselmouth.praat.call(tgt, "Get label of interval", i, j)
                    items.append((start, end, label))
            else:
                nitems = parselmouth.praat.call(tgt, "Get number of points", i)
                for j in range(1, nitems + 1):
                    time_val = float(
                        parselmouth.praat.call(tgt, "Get time of point", i, j)
                    )
                    label = parselmouth.praat.call(tgt, "Get label of point", i, j)
                    items.append((time_val, time_val, label))
            tiers.append((tier_name, is_interval, items))
    result[str(fpath)] = (tmin, tmax, tiers)
    if len(files) > 1:
        return result
    return next(iter(result.values()))


METHODS: list[BenchmarkMethod] = [
    BenchmarkMethod(
        name="pytextgrid",
        to_dataframe=pytextgrid_to_dataframe,
        to_structured=pytextgrid_to_structured,
    ),
    BenchmarkMethod(
        name="textgrid",
        to_dataframe=textgridpkg_to_dataframe,
        to_structured=textgridpkg_to_structured,
    ),
    BenchmarkMethod(
        name="parselmouth",
        to_dataframe=parselmouth_to_dataframe,
        to_structured=parselmouth_to_structured,
    ),
]


def time_callable(func: Callable[[], object], repeats: int) -> list[float]:
    durations: list[float] = []
    for _ in range(repeats):
        start = time.perf_counter()
        func()
        durations.append(time.perf_counter() - start)
    return durations


def run_benchmarks(scenarios: Iterable[Scenario], repeats: int) -> pd.DataFrame:
    records: list[dict[str, object]] = []
    for scenario in scenarios:
        files = _ensure_files(scenario.files)
        for method in METHODS:
            df_times = time_callable(lambda: method.to_dataframe(files), repeats)
            for run_idx, delta in enumerate(df_times, start=1):
                records.append(
                    {
                        "scenario": scenario.name,
                        "description": scenario.description,
                        "method": method.name,
                        "output": "dataframe",
                        "run": run_idx,
                        "seconds": delta,
                    }
                )
            struct_times = time_callable(lambda: method.to_structured(files), repeats)
            for run_idx, delta in enumerate(struct_times, start=1):
                records.append(
                    {
                        "scenario": scenario.name,
                        "description": scenario.description,
                        "method": method.name,
                        "output": "structured",
                        "run": run_idx,
                        "seconds": delta,
                    }
                )
    return pd.DataFrame.from_records(records)


def summarise(results: pd.DataFrame) -> pd.DataFrame:
    summary = results.groupby(
        ["scenario", "description", "method", "output"], as_index=False
    ).agg(
        mean=("seconds", "mean"),
        std=("seconds", "std"),
        min=("seconds", "min"),
        max=("seconds", "max"),
    )
    return summary


def plot_results(summary: pd.DataFrame, figure_path: Path) -> None:
    outputs = ["dataframe", "structured"]
    methods = [method.name for method in METHODS]
    scenarios = list(summary["scenario"].unique())
    width = 0.25
    fig, axes = plt.subplots(1, len(outputs), figsize=(12, 5), sharey=True)
    axes = np.atleast_1d(axes)
    for ax, output in zip(axes, outputs):
        subset = summary[summary["output"] == output]
        for idx, scenario in enumerate(scenarios):
            scenario_data = subset[subset["scenario"] == scenario]
            for midx, method in enumerate(methods):
                row = scenario_data[scenario_data["method"] == method]
                if row.empty:
                    continue
                mean_val = float(row["mean"].iloc[0])
                std_raw = row["std"].iloc[0]
                std_val = float(std_raw) if pd.notna(std_raw) else 0.0
                x_pos = idx + (midx - (len(methods) - 1) / 2.0) * width
                ax.bar(
                    x_pos,
                    mean_val,
                    width=width,
                    yerr=std_val,
                    label=method if idx == 0 else "",
                )
        ax.set_title(f"{output.capitalize()} output")
        ax.set_xticks(range(len(scenarios)))
        ax.set_xticklabels(scenarios, rotation=20, ha="right")
        ax.set_ylabel("Seconds")
        ax.set_xlim(-0.5, len(scenarios) - 0.5)
        ax.grid(axis="y", linestyle="--", alpha=0.3)
    axes[0].legend(title="Method")
    fig.tight_layout()
    figure_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(figure_path, dpi=150)
    plt.close(fig)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark TextGrid conversion libraries"
    )
    parser.add_argument(
        "--repeats", type=int, default=3, help="Number of repetitions per measurement"
    )
    parser.add_argument(
        "--many-files",
        type=int,
        default=200,
        help="Number of small TextGrid files to generate",
    )
    parser.add_argument(
        "--many-interval-tiers",
        type=int,
        default=1,
        help="Interval tiers per small TextGrid file",
    )
    parser.add_argument(
        "--many-point-tiers",
        type=int,
        default=1,
        help="Point tiers per small TextGrid file",
    )
    parser.add_argument(
        "--many-interval-items",
        type=int,
        default=20,
        help="Interval items per tier in small TextGrid files",
    )
    parser.add_argument(
        "--many-point-items",
        type=int,
        default=20,
        help="Point items per tier in small TextGrid files",
    )
    parser.add_argument(
        "--many-step",
        type=float,
        default=0.05,
        help="Time step used to build small TextGrid files",
    )
    parser.add_argument(
        "--large-interval-tiers",
        type=int,
        default=2,
        help="Interval tiers for the large TextGrid file",
    )
    parser.add_argument(
        "--large-point-tiers",
        type=int,
        default=2,
        help="Point tiers for the large TextGrid file",
    )
    parser.add_argument(
        "--large-interval-items",
        type=int,
        default=5000,
        help="Interval items per tier in the large TextGrid file",
    )
    parser.add_argument(
        "--large-point-items",
        type=int,
        default=5000,
        help="Point items per tier in the large TextGrid file",
    )
    parser.add_argument(
        "--large-step",
        type=float,
        default=0.01,
        help="Time step used to build the large TextGrid file",
    )
    parser.add_argument(
        "--figure",
        type=Path,
        default=Path("benchmark_results.png"),
        help="Path to save the comparison plot",
    )
    parser.add_argument(
        "--csv",
        type=Path,
        default=None,
        help="Optional path to export aggregated results as CSV",
    )
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=None,
        help="Keep generated TextGrid data in this directory instead of a temporary folder",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    temp_dir: TemporaryDirectory | None = None
    if args.data_dir:
        base_dir = args.data_dir
        base_dir.mkdir(parents=True, exist_ok=True)
    else:
        temp_dir = TemporaryDirectory()
        base_dir = Path(temp_dir.name)
    try:
        scenarios = [
            create_many_small_files(
                base_dir,
                file_count=args.many_files,
                interval_tiers=args.many_interval_tiers,
                point_tiers=args.many_point_tiers,
                interval_items=args.many_interval_items,
                point_items=args.many_point_items,
                step=args.many_step,
            ),
            create_single_large_file(
                base_dir,
                interval_tiers=args.large_interval_tiers,
                point_tiers=args.large_point_tiers,
                interval_items=args.large_interval_items,
                point_items=args.large_point_items,
                step=args.large_step,
            ),
        ]
        results = run_benchmarks(scenarios, repeats=max(args.repeats, 1))
        summary = summarise(results)
        print("Raw measurements:")
        print(results.to_string(index=False))
        print("\nAggregated summary:")
        print(summary.to_string(index=False))
        if args.csv:
            args.csv.parent.mkdir(parents=True, exist_ok=True)
            summary.to_csv(args.csv, index=False)
        plot_results(summary, Path(args.figure))
        print(f"\nFigure saved to {args.figure}")
        if args.data_dir:
            print(f"Generated TextGrid files kept in {args.data_dir}")
    finally:
        if temp_dir is not None:
            temp_dir.cleanup()


if __name__ == "__main__":
    main()
