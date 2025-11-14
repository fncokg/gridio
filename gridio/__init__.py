"""High-level helpers for converting Praat TextGrid files.

These wrappers expose the Rust-backed parsing and serialization
implemented in :mod:`gridio.gridio` and provide convenient
Pythonic utilities for turning TextGrid content into tabular or nested
data structures and writing them back to disk.
"""

# Here we provide some wrapper functions for Rust implementations
from .gridio import (
    textgrid2vectors as rc_tg2vecs,
    textgrids2vectors as rc_tgs2vecs,
    textgrid2data as rc_tg2data,
    textgrids2data as rc_tgs2data,
    data2textgrid as rc_data2tg,
    vectors2textgrid as rc_vecs2tg,
)
from typing import Union, Literal, Any, Optional, TYPE_CHECKING
from pathlib import Path
import numpy as np

if TYPE_CHECKING:  # pragma: no cover - for type checkers only
    from .textgrid import (
        TextGrid,
        Tier,
        IntervalTier,
        PointTier,
        IntervalItem,
        PointItem,
    )


__version__ = "1.0.5"


def _file_name(
    file: Union[str, list[str], Path, list[Path]], file_name_func: Optional[Any] = None
) -> Union[str, list[str]]:
    """Return normalized file name(s) for a single path or a list of paths."""
    if file_name_func is None:
        file_name_func = lambda f: str(f)
    if isinstance(file, (str, Path)):
        return file_name_func(file)
    elif isinstance(file, list):
        return list(map(file_name_func, file))
    else:
        raise TypeError("file must be str/Path or list of them")


def _dispatch_files(
    file: Union[str, list[str], Path, list[Path]],
    func_single,
    func_multiple,
    **kwargs,
):
    """Dispatch to the single- or multi-file Rust binding based on input type."""
    if isinstance(file, (str, Path)):
        result = func_single(str(file), **kwargs)
    elif isinstance(file, list):
        result = func_multiple(list(map(str, file)), **kwargs)
    else:
        raise TypeError("file must be str/Path or list of them")

    return result


def textgrid_to_df(
    file: Union[str, list[str], Path, list[Path]],
    strict: bool = False,
    file_type: str = "auto",
    file_name_column: Optional[bool] = None,
    file_name_func: Optional[Any] = None,
    backend: Literal["pandas", "polars"] = "pandas",
):
    """Parse TextGrid files into a DataFrame-like structure.

    Parameters
    ----------
    file:
        Path to a single TextGrid file or an iterable of paths.
    strict:
        When ``True`` the parser raises on malformed structures instead of
        attempting a best-effort conversion.
    file_type:
        ``"short"``, ``"long"``, or ``"auto"`` to let the backend infer the
        dialect.
    file_name_column:
        Force inclusion (``True``) or exclusion (``False``) of a filename column.
        Defaults to ``True`` for multi-file inputs and ``False`` otherwise.
    file_name_func:
        Optional callable used to transform each filename before it is written
        to the DataFrame. The callable receives the original path object and
        must return a string.
    backend:
        ``"pandas"`` yields a :class:`pandas.DataFrame`; ``"polars"`` yields a
        :class:`polars.DataFrame`.

    Returns
    -------
    pandas.DataFrame or polars.DataFrame
        Tabular representation containing ``tmin``, ``tmax``, ``label``, ``tier``
        and interval flags; optionally includes ``filename``.

    Examples
    --------
    >>> df = textgrid_to_df("data/short_format.TextGrid")
    >>> df.head().to_dict(orient="records")  # doctest: +SKIP
    [{'tmin': 0.0, 'tmax': 0.25, 'label': 'sil', 'tier': 'phones',
      'is_interval': True},
     {'tmin': 0.25, 'tmax': 0.53, 'label': 's', 'tier': 'phones',
      'is_interval': True}]

    The primary columns are ``tmin``, ``tmax``, ``label``, ``tier``,
    ``is_interval``, and (for multi-file inputs) ``filename``.

    See Also
    --------
    df_to_textgrid : Persist the DataFrame back to a TextGrid file.
    """
    if file_name_column is None:
        file_name_column = isinstance(file, list)

    vectors = _dispatch_files(
        file,
        func_single=lambda f: rc_tg2vecs(f, strict=strict, file_type=file_type),
        func_multiple=lambda fs: rc_tgs2vecs(fs, strict=strict, file_type=file_type),
    )

    tmins, tmaxs, labels, tiers, is_intervals = vectors[:5]

    data = {
        "tmin": tmins,
        "tmax": tmaxs,
        "label": np.array(labels, dtype=np.str_),
        "tier": np.array(tiers, dtype=np.str_),
        "is_interval": is_intervals,
    }

    if file_name_column:
        file_names = _file_name(file, file_name_func=file_name_func)
        if isinstance(file, (str, Path)):
            file_names = np.repeat(file_names, len(tiers))
        else:
            file_ids = vectors[5]
            file_names = np.array(file_names, dtype=np.str_)[file_ids]
        data["filename"] = file_names

    if backend == "pandas":
        import pandas as pd

        df = pd.DataFrame(data, copy=False)
    elif backend == "polars":
        import polars as pl

        df = pl.DataFrame(data)
    else:
        raise ValueError("backend must be 'pandas' or 'polars'")
    return df


def textgrid_to_data(
    file: Union[str, list[str], Path, list[Path]],
    strict: bool = False,
    file_name_func: Optional[Any] = None,
    file_type: str = "auto",
):
    """Parse TextGrid files into nested data resembling the Rust output.

    Parameters
    ----------
    file:
        Path to a single TextGrid file or an iterable of paths.
    strict:
        When ``True`` enforce strict parsing.
    file_name_func:
        Optional callable used to transform each filename key in the returned
        dictionary for multi-file inputs.
    file_type:
        ``"short"``, ``"long"`` or ``"auto"`` to control dialect detection.

    Returns
    -------
    tuple or dict[str, tuple]
        The raw structured data from the Rust bindings. For multiple files a
        dictionary keyed by filename is returned.

    Examples
    --------
    >>> data = textgrid_to_data("data/short_format.TextGrid")
    >>> round(data[0], 2), round(data[1], 2)
    (0.0, 2.43)
    >>> data[2][0]
    ('phones', True, [(0.0, 0.25, 'sil'), (0.25, 0.53, 's')])

    The tuple layout is ``(tmin, tmax, tiers)``. Each tier entry is
    ``(name, is_interval, items)`` where ``items`` is an ordered list of
    ``(start, end, label)`` tuples. Point tiers repeat their timestamp for
    ``start`` and ``end`` so the shape remains consistent with interval tiers.

    With multiple input files the function returns a dictionary mapping the
    normalised filename (after applying ``file_name_func`` when provided) to the
    same tuple structure.

    See Also
    --------
    data_to_textgrid : Convert the tuple structure back into a TextGrid file.
    """

    data = _dispatch_files(
        file,
        func_single=lambda f: rc_tg2data(f, strict=strict, file_type=file_type),
        func_multiple=lambda fs: rc_tgs2data(fs, strict=strict, file_type=file_type),
    )
    if isinstance(file, (str, Path)):
        return data
    file_names = _file_name(file, file_name_func=file_name_func)
    return {file_name: file_data for file_name, file_data in zip(file_names, data)}


def df_to_textgrid(
    df: Any,
    out_file: str,
    tmin: Optional[float] = None,
    tmax: Optional[float] = None,
    file_type: str = "long",
):
    """Serialize a tabular representation back into a TextGrid file.

    Parameters
    ----------
    df:
        DataFrame-like object exposing ``tmin``, ``tmax``, ``label``, ``tier``
        and ``is_interval`` columns.
    out_file:
        Destination path for the emitted TextGrid file.
    tmin, tmax:
        Optional overrides for the global bounds written to the file.
    file_type:
        Dialect to emit (``"short"`` or ``"long"``).

    Examples
    --------
    >>> df = textgrid_to_df("data/short_format.TextGrid")
    >>> df_to_textgrid(df, "out.TextGrid", file_type="short")  # doctest: +SKIP

    The converter expects the same column layout described in
    :func:`textgrid_to_df`.

    See Also
    --------
    textgrid_to_df : Parse TextGrid files into the expected DataFrame format.
    """
    tmins = df["tmin"].tolist()
    tmaxs = df["tmax"].tolist()
    labels = df["label"].tolist()
    tiers = df["tier"].tolist()
    is_intervals = df["is_interval"].tolist()
    rc_vecs2tg(
        tmins,
        tmaxs,
        labels,
        tiers,
        is_intervals,
        tmin,
        tmax,
        out_file,
        file_type=file_type,
    )


def data_to_textgrid(
    data: Any,
    out_file: str,
    file_type: str = "long",
):
    """Write nested tier data to a TextGrid file using the Rust backend.

    Parameters
    ----------
    data:
        Tuple of ``(tmin, tmax, tiers)`` as returned by :func:`textgrid_to_data`.
    out_file:
        Destination path for the serialized TextGrid.
    file_type:
        Dialect to emit (``"short"`` or ``"long"``).

    Examples
    --------
    >>> data = textgrid_to_data("data/short_format.TextGrid")
    >>> data_to_textgrid(data, "out.TextGrid", file_type="long")  # doctest: +SKIP

    Refer to :func:`textgrid_to_data` for details on the expected tuple layout.

    See Also
    --------
    textgrid_to_data : Produce the tuple structure consumed by this helper.
    """
    tmin, tmax, tiers = data
    rc_data2tg(tiers, tmin, tmax, out_file, file_type=file_type)


__all__ = [
    "textgrid_to_df",
    "textgrid_to_data",
    "df_to_textgrid",
    "data_to_textgrid",
    "TextGrid",
    "Tier",
    "IntervalTier",
    "PointTier",
    "IntervalItem",
    "PointItem",
]


_LAZY_TEXTGRID_EXPORTS = {
    "TextGrid",
    "Tier",
    "IntervalTier",
    "PointTier",
    "IntervalItem",
    "PointItem",
}


def __getattr__(name: str):
    if name in _LAZY_TEXTGRID_EXPORTS:
        from importlib import import_module

        module = import_module(".textgrid", __name__)
        value = getattr(module, name)
        globals()[name] = value  # cache for future lookups
        return value
    raise AttributeError(f"module '{__name__}' has no attribute '{name}'")
