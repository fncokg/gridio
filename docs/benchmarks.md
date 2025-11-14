# Benchmarks

This document presents performance benchmarks comparing three popular Python libraries for parsing Praat TextGrid files:

1. **gridio** (this project, backed by Rust bindings)
2. **textgrid** (pure Python implementation)
3. **parselmouth** (Python bridge to Praat''s C++ core)

## Overview

The benchmark suite evaluates two common workflows:

- **construct**: Parse TextGrid files and construct in-memory objects
- **to_df**: Parse TextGrid files and convert them to Pandas DataFrames

These tasks represent typical use cases in phonetic research, where researchers need to either work with TextGrid objects programmatically or analyze annotation data using DataFrame operations.

## Implementation Details

### gridio

```python
import gridio

# Task 1: construct - Parse and create TextGrid objects
textgrids = gridio.TextGrid.from_file(file_list)

# Task 2: to_df - Parse and convert to DataFrame
df = gridio.textgrid_to_df(file_list, strict=False, file_type="long")
```

The `gridio.TextGrid.from_file()` method accepts either a single file path or a list of paths, enabling efficient batch processing with Rust-powered parallelization.

### textgrid (pure Python)

```python
import textgrid
import pandas as pd

# Task 1: construct
textgrids = [textgrid.TextGrid.fromFile(f) for f in file_list]

# Task 2: to_df
rows = []
for fname, tg in zip(file_list, textgrids):
    for tier in tg.tiers:
        is_interval = isinstance(tier, textgrid.IntervalTier)
        for item in tier:
            if is_interval:
                tmin, tmax, label = item.minTime, item.maxTime, item.mark
            else:
                tmin, tmax, label = item.time, item.time, item.mark
            rows.append({
                "tmin": tmin,
                "tmax": tmax,
                "label": label,
                "tier": tier.name,
                "is_interval": is_interval,
                "filename": fname,
            })
df = pd.DataFrame(rows)
```

### parselmouth

```python
from functools import partial
import parselmouth
import pandas as pd

# Task 1: construct
data_list = []
for fname in file_list:
    tgt = parselmouth.TextGrid.read(str(fname))
    Praat = partial(parselmouth.praat.call, tgt)
    
    tgt_data = {
        "xmin": Praat("Get start time"),
        "xmax": Praat("Get end time"),
        "tiers": [],
    }
    ntier = Praat("Get number of tiers")
    for tier_idx in range(1, ntier + 1):
        tier = {
            "name": Praat("Get tier name", tier_idx),
            "is_interval": Praat("Is interval tier", tier_idx),
            "items": [],
        }
        if tier["is_interval"]:
            nitems = Praat("Get number of intervals", tier_idx)
            for j in range(1, nitems + 1):
                tier["items"].append({
                    "tmin": Praat("Get start time of interval", tier_idx, j),
                    "tmax": Praat("Get end time of interval", tier_idx, j),
                    "label": Praat("Get label of interval", tier_idx, j),
                })
        else:
            nitems = Praat("Get number of points", tier_idx)
            for j in range(1, nitems + 1):
                tier["items"].append({
                    "point": Praat("Get time of point", tier_idx, j),
                    "label": Praat("Get label of point", tier_idx, j),
                })
        tgt_data["tiers"].append(tier)
    data_list.append(tgt_data)

# Task 2: to_df (similar nested loop structure)
```

## Benchmark Configuration & Test Environment

The test corpus simulates a realistic scenario with moderate-sized annotation files, consists of **5,000 TextGrid files**, each containing **2 interval tiers** with 50 intervals each, **2 point tiers** with 50 points each, and stored in Long format TextGrid files.

Benchmarks were conducted on a laptop with the following specifications:

- **OS**: Windows 11 (25H2)
- **CPU**: 12th Gen Intel® Core™ i5-12500H
- **RAM**: 16 GB
- **Python**: 3.12.7 (64-bit)
- **gridio**: Latest version (Rust-backed)
- **textgrid**: 1.6.1
- **parselmouth**: 0.4.5
- **pandas**: 2.2.3

Each benchmark was repeated **5 times** with a warm-up run to eliminate cold-start effects.

## Results

### Summary Statistics

The table below shows mean execution time across 5 runs, with standard deviation and the speedup factor relative to the fastest implementation:

| Package     | Task      | Mean (s) | Std Dev (s) | Speedup |
| ----------- | --------- | -------- | ----------- | ------- |
| gridio      | construct | 0.984    | 0.042       | 1.0x    |
| textgrid    | construct | 8.555    | 0.031       | 8.7x    |
| parselmouth | construct | 206.68   | 3.39        | 210.0x  |
| gridio      | to_df     | 1.264    | 0.014       | 1.0x    |
| textgrid    | to_df     | 10.143   | 0.945       | 8.0x    |
| parselmouth | to_df     | 220.11   | 5.23        | 174.1x  |

### Visualization

![Benchmark Results](https://github.com/fncokg/gridio/blob/master/benchmarks/results/benchmark_plot.png?raw=true)

*Figure: Comparison of execution times for parsing 5,000 TextGrid files. The bars show mean execution time with error bars indicating standard deviation. Numbers on bars indicate absolute time (seconds) and relative speedup (x) compared to the fastest method. The x axis has been sqrt-scaled for clarity.*

### Key Findings

`gridio` dominates both tasks. It's ~8x faster than `textgrid` and over 170x faster than `parselmouth`. You should prefer `gridio` for large-scale TextGrid processing due to its superior performance.

## Running the Benchmark

To reproduce these results:

```bash
cd benchmarks
python benchmark.py
```

Modifications to the benchmark parameters (e.g., number of files, tiers, intervals) can be made in `benchmark.py`.

To visualize the results:

```r
# Open plot.Rmd in VSCode/RStudio and run the code chunks
# Or use the R terminal:
cd benchmarks
Rscript -e "rmarkdown::render('plot.Rmd')"
```

This generates `benchmark_plot.png` in the `benchmarks/results/` directory.
