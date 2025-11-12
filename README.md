# pytextgrid

Rust-powered TextGrid parsing for Python. `pytextgrid` offers user-friendly APIs with high performance for manipulating Praat TextGrid files.

## Why pytextgrid?

- **High Performance** – Built with Rust, `pytextgrid` is designed for speed and efficiency, outperforming pure Python implementations as well as bindings to Praat's C++ core.
- **Flexible APIs** – Whether you prefer working with DataFrames, object-oriented structures, or JSON-like data, `pytextgrid` has you covered.

## Quick Start

### I want simplicity

No structures. No classes. Just load and save DataFrames.

```python
from pytextgrid import textgrid_to_df, df_to_textgrid

df = textgrid_to_df("data/short_format.TextGrid")
print(df.head())

df_to_textgrid(df, "output.TextGrid", file_type="short")
```

### I want full control

You can manipulate TextGrid files with a OOP-style API.

```python
from pytextgrid import TextGrid, Tier, IntervalItem

tg: TextGrid = TextGrid.from_file("data/long_format.TextGrid")
phones: Tier = tg.get_tier("phone")
new_item: IntervalItem = IntervalItem(1.23, 1.45, "ah")
phones.insert_item(new_item, index=0)

tg.save("edited.TextGrid", file_type="long")
```

*No worry about performance, all of those OOP objects are lazy created from raw data only when accessed.*

### You can even work with a JSON-like data structure.

With `textgrid_to_data` and `data_to_textgrid`, convert between TextGrid files and nested lists/dicts. They're easy to serialize (e.g., to JSON) and manipulate programmatically.

```python
from pytextgrid import textgrid_to_data, data_to_textgrid

data = textgrid_to_data("data/long_format.TextGrid")
print(data[0], data[1])  # global tmin/tmax
first_tier = data[2][0]
print(first_tier[0], first_tier[2][:2])

data_to_textgrid(data, "copy.TextGrid")
```


## Install

```bash
pip install pytextgrid
# or from source
maturin develop
```

## Benchmarks

We benchmark `pytextgrid` against two popular TextGrid parsing libraries: `textgrid` (a pure Python implementation) and `parselmouth` (Python bindings for Praat). 

The benchmarks focus on two common tasks: 
- constructing in-memory TextGrid objects
- converting TextGrids to Pandas DataFrames

The results are summarized below:

| Package        | Task      | Mean (s) | Std Dev (s) | Speedup |
| -------------- | --------- | -------- | ----------- | ------- |
| **pytextgrid** | construct | 0.984    | 0.042       | 1.0x    |
| textgrid       | construct | 8.555    | 0.031       | 8.7x    |
| parselmouth    | construct | 206.68   | 3.39        | 210.0x  |
| **pytextgrid** | to_df     | 1.264    | 0.014       | 1.0x    |
| textgrid       | to_df     | 10.143   | 0.945       | 8.0x    |
| parselmouth    | to_df     | 220.11   | 5.23        | 174.1x  |

![Benchmark Results](benchmarks/results/benchmark_plot.png)
