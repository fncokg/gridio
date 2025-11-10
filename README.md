# pytextgrid

Rust-powered TextGrid parsing for Python. `pytextgrid` bridges a high-performance
Rust core with a friendly Python API so you can ingest, transform, and emit
Praat TextGrid files with minimal overhead.

## Why pytextgrid?

- 🚀 **Speed first** – The heavy lifting lives in Rust, making conversions dozens
  to hundreds of times faster than pure-Python alternatives.
- 🧰 **Familiar tools** – Convert TextGrids straight into Pandas/Polars
  DataFrames, or work with a lightweight tuple structure for custom pipelines.
- 🧱 **OOP convenience** – Manipulate tiers and items via a Pythonic
  `TextGrid` class without sacrificing performance.

## Quick Examples

### DataFrame round-trip

```python
from pytextgrid import textgrid_to_df, df_to_textgrid

df = textgrid_to_df("data/short_format.TextGrid")
print(df.head())

df_to_textgrid(df, "roundtrip.TextGrid", file_type="short")
```

### Structured tuple workflow

```python
from pytextgrid import textgrid_to_data, data_to_textgrid

data = textgrid_to_data("data/long_format.TextGrid")
print(data[0], data[1])  # global tmin/tmax
first_tier = data[2][0]
print(first_tier[0], first_tier[2][:2])

data_to_textgrid(data, "copy.TextGrid")
```

### Object-oriented editing

```python
from pytextgrid import TextGrid, IntervalItem

tg = TextGrid.from_file("data/long_format.TextGrid")
phones = tg.get_tier("phone")
phones.insert_item(IntervalItem(0.0, 0.05, "noise"), index=0)

tg.save("edited.TextGrid", file_type="long")
```

## Install

```bash
pip install pytextgrid
# or from source
maturin develop
```

## Benchmarks

Three representative scenarios (many small files, single large file, many large
files) show `pytextgrid` leading the pack. Detailed commands and charts live in
`docs/benchmarks.md`; as a teaser:

| Scenario          | textgrid (ms) | parselmouth (ms) | pytextgrid (ms) |
| ----------------- | ------------: | ---------------: | --------------: |
| many small files  |           140 |             1950 |          **30** |
| single large file |           330 |             6600 |          **50** |
| many large files  |          3220 |            68800 |         **760** |

## License

Licensed under the MIT License. See `LICENSE` for details.
