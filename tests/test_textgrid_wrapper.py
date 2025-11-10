import os
from pathlib import Path
from pytextgrid.textgrid import (
    TextGrid,
    IntervalTier,
    PointTier,
    IntervalItem,
    PointItem,
)

DATA_DIR = Path(__file__).parent.parent / "data"
LONG_TG = DATA_DIR / "long_format.TextGrid"
SHORT_TG = DATA_DIR / "short_format.TextGrid"


def test_read_single_long():
    tg = TextGrid.from_file(LONG_TG)
    assert isinstance(tg, TextGrid)
    # expect 5 tiers in sample file
    assert tg.get_tier(0).name == "phone"
    assert tg.get_tier("points").is_interval is False


def test_read_single_short():
    tg = TextGrid.from_file(SHORT_TG)
    assert isinstance(tg, TextGrid)
    # 2 tiers in short example
    assert tg.get_tier("word").is_interval is True


def test_add_remove_tier_and_items(tmp_path):
    tg = TextGrid()
    phone = IntervalTier("phone")
    phone.insert_item(IntervalItem(0.0, 0.5, "a"))
    phone.insert_item(IntervalItem(0.5, 1.0, "b"))
    tg.add_tier(phone)

    pts = PointTier("points")
    pts.insert_item(PointItem(0.25, "m"))
    tg.add_tier(pts)

    assert tg.get_tier("phone").is_interval
    assert tg.get_tier("phone").nitems == 2
    assert not tg.get_tier("points").is_interval

    # remove one item
    t = tg.get_tier("phone")
    assert t.get_item(0).label == "a"

    # save & reload
    out_file = tmp_path / "out.TextGrid"
    tg.tmin, tg.tmax = 0.0, 1.0
    tg.save(str(out_file), file_type="long")
    assert out_file.exists()

    tg2 = TextGrid.from_file(out_file)
    assert tg2.get_tier("phone").get_item(0).label == "a"


def test_batch_read():
    res = TextGrid.from_file([LONG_TG, SHORT_TG])
    assert isinstance(res, dict)
    assert set(res.keys()) == {str(LONG_TG), str(SHORT_TG)}
