from pathlib import Path

import pytest

from pytextgrid import df_to_textgrid, textgrid_to_df
from pytextgrid.textgrid import TextGrid

DATA_DIR = Path(__file__).parent.parent / "data"
LONG_TG = DATA_DIR / "long_format.TextGrid"


@pytest.mark.parametrize("backend", ["pandas"])
def test_textgrid_to_df_has_expected_columns(backend):
    df = textgrid_to_df(LONG_TG, backend=backend)
    assert not df.empty
    expected_columns = {"tmin", "tmax", "label", "tier", "is_interval"}
    assert expected_columns.issubset(df.columns), df.columns


def test_df_to_textgrid_roundtrip(tmp_path):
    df = textgrid_to_df(LONG_TG)
    out_file = tmp_path / "roundtrip.TextGrid"
    tmin = df["tmin"].min()
    tmax = df["tmax"].max()

    df_to_textgrid(df, str(out_file), tmin=tmin, tmax=tmax, file_type="long")
    assert out_file.exists()

    original = TextGrid.from_file(LONG_TG)
    written = TextGrid.from_file(out_file)
    assert written.ntiers == original.ntiers

    for tier_index in range(original.ntiers):
        orig_tier = original.get_tier(tier_index)
        new_tier = written.get_tier(tier_index)

        assert new_tier.name == orig_tier.name
        assert new_tier.is_interval == orig_tier.is_interval
        assert new_tier.nitems == orig_tier.nitems

        for item_index in range(orig_tier.nitems):
            assert (
                new_tier.get_item(item_index).data
                == orig_tier.get_item(item_index).data
            )
