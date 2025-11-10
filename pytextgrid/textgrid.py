# This is an OOP wrapper
from pytextgrid import textgrid_to_data, data_to_textgrid
from typing import Optional, Union, Iterable, Iterator, Tuple, List, Dict, Any, Literal
from pathlib import Path


class IntervalItem:
    """Lightweight container for a single interval tier entry.

    Parameters
    ----------
    tmin, tmax : float
        Start and end boundaries in seconds.
    label : str
        Text label associated with the interval.
    """

    def __init__(self, tmin, tmax, label):
        self.tmin = tmin
        self.tmax = tmax
        self.label = label

    @property
    def data(self):
        """Return the tuple representation consumed by the Rust backend."""
        return (self.tmin, self.tmax, self.label)


class PointItem:
    """Container for a point tier entry stored at a single timestamp.

    Parameters
    ----------
    time : float
        Absolute time position in seconds.
    label : str
        Text label associated with the point.
    """

    def __init__(self, time, label):
        self.time = time
        self.label = label

    @property
    def data(self):
        """Represent the point as ``(time, time, label)`` for uniform storage."""
        return (self.time, self.time, self.label)


class Tier:
    """Mutable collection of TextGrid items with a shared tier name.

    Parameters
    ----------
    name : str
        Tier identifier as stored in the TextGrid.
    is_interval : bool
        ``True`` for interval tiers, ``False`` for point tiers.
    """

    def __init__(self, name, is_interval):
        self.name = name
        self.is_interval = is_interval
        self._items = []

    def insert_item(self, item, index: int = -1):
        """Insert a new item at ``index`` (append when ``index`` is ``-1``)."""
        if index == -1:
            index = len(self._items)
        self._items.insert(index, item.data)

    def remove_item(self, index: int):
        """Remove the item at ``index`` from the tier."""
        self._items.pop(index)

    @property
    def nitems(self) -> int:
        """Number of stored items."""
        return len(self._items)

    @property
    def data(self):
        """Return the tuple ``(name, is_interval, items)`` used by the bindings."""
        return (self.name, self.is_interval, self._items)

    @staticmethod
    def _from_data(data):
        """Instantiate an appropriate tier subclass from backend data."""
        name, is_interval, items = data
        if is_interval:
            tier = IntervalTier(name)
        else:
            tier = PointTier(name)
        tier._items = items
        return tier


class PointTier(Tier):
    """Tier containing point items at discrete timestamps."""

    def __init__(self, name):
        super().__init__(name, False)

    def get_item(self, index: int) -> PointItem:
        """Return the :class:`PointItem` stored at ``index``."""
        item_data = self._items[index]
        return PointItem(item_data[0], item_data[2])


class IntervalTier(Tier):
    """Tier containing interval items spanning start and end times."""

    def __init__(self, name):
        super().__init__(name, True)

    def get_item(self, index: int) -> IntervalItem:
        """Return the :class:`IntervalItem` stored at ``index``."""
        item_data = self._items[index]
        return IntervalItem(item_data[0], item_data[1], item_data[2])


class TextGrid:
    """In-memory representation of a Praat TextGrid document.

    Parameters
    ----------
    tmin, tmax : float, optional
        Global bounds for all tiers. They are preserved when saving if not
        overridden.
    """

    def __init__(self, tmin=None, tmax=None):
        self.tmin = tmin
        self.tmax = tmax
        self._tiers = []

    def _name2id(self, tier_name: str) -> Optional[int]:
        for i, tier in enumerate(self._tiers):
            if tier[0] == tier_name:
                return i
        return None

    def _tier_route(self, indexer=None, tier_name=None, tier_id=None) -> Optional[int]:
        if not indexer is None:
            if isinstance(indexer, int):
                tier_id = indexer
            elif isinstance(indexer, str):
                tier_name = indexer
            else:
                raise TypeError("indexer must be int or str")
        if not tier_name is None:
            tier_id = self._name2id(tier_name)
        return tier_id

    def get_tier(self, indexer=None, tier_name=None, tier_id=None) -> Optional[Tier]:
        """Return a tier by numeric index or name, preserving subclass type."""
        tier_id = self._tier_route(
            indexer=indexer, tier_name=tier_name, tier_id=tier_id
        )
        return Tier._from_data(self._tiers[tier_id])

    def add_tier(self, tier: Tier, where: int = -1):
        """Insert ``tier`` at ``where`` (append when ``where`` is ``-1``)."""
        if where == -1:
            where = len(self._tiers)
        self._tiers.insert(where, tier.data)

    def remove_tier(self, tier_name=None, tier_id=None):
        """Remove a tier, looking it up by name or numeric index."""
        tier_id = self._tier_route(tier_name=tier_name, tier_id=tier_id)
        if tier_id is None:
            return
        self._tiers.pop(tier_id)

    @property
    def ntiers(self) -> int:
        """Total number of tiers in the TextGrid."""
        return len(self._tiers)

    @property
    def data(self):
        """Return the backend-compatible tuple ``(tmin, tmax, tiers)``."""
        return (self.tmin, self.tmax, self._tiers)

    def save(
        self,
        out_file: str,
        file_type: str = "long",
    ):
        """Write the TextGrid to ``out_file`` using the Rust serializer."""
        data_to_textgrid(self.data, out_file, file_type=file_type)

    @staticmethod
    def _from_data(data):
        """Build a :class:`TextGrid` instance from ``textgrid_to_data`` output."""
        tmin, tmax, tiers = data
        tg = TextGrid(tmin, tmax)
        tg._tiers = tiers
        return tg

    @staticmethod
    def from_file(
        file: Union[str, Path, list[str], list[Path]],
        strict: bool = False,
        file_type: str = "auto",
        file_name_func: Optional[Any] = None,
    ) -> Union["TextGrid", Dict[str, "TextGrid"]]:
        """Read TextGrid files and wrap them in :class:`TextGrid` objects."""
        tg_data = textgrid_to_data(
            file,
            strict=strict,
            file_type=file_type,
            file_name_func=file_name_func,
        )
        if isinstance(file, (str, Path)):
            return TextGrid._from_data(tg_data)
        else:
            return {fname: TextGrid._from_data(data) for fname, data in tg_data.items()}


__all__ = [
    "IntervalItem",
    "PointItem",
    "Tier",
    "PointTier",
    "IntervalTier",
    "TextGrid",
]
