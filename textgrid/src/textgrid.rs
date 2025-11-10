use std::io::{Error, ErrorKind, Result};

const TIME_EPSILON: f64 = 1e-6;
pub struct Item {
    pub tmin: f64,
    pub tmax: f64,
    pub label: String,
}

pub struct Tier {
    pub name: String,
    pub size: usize,
    pub items: Vec<Item>,
    pub interval_tier: bool,
    pub tmin: f64,
    pub tmax: f64,
}

pub struct TextGrid {
    pub tmin: f64,
    pub tmax: f64,
    pub size: usize,
    pub name: String,
    pub tiers: Vec<Tier>,
}

impl Item {
    pub fn new() -> Self {
        Item {
            tmin: 0.0,
            tmax: 0.0,
            label: String::new(),
        }
    }
}

pub fn data_error(msg: &str) -> Error {
    Error::new(ErrorKind::InvalidData, msg)
}

pub fn assert_valid_time_bounds(tmin: f64, tmax: f64, where_msg: &str) -> Result<()> {
    if tmin < 0.0 || tmax <= 0.0 {
        return Err(data_error(&format!(
            "Time bounds should be non-negative in {}",
            where_msg
        )));
    }
    if tmax - tmin <= TIME_EPSILON {
        return Err(data_error(&format!(
            "tmin should be less than tmax in {}",
            where_msg
        )));
    }
    Ok(())
}

impl Tier {
    pub fn new() -> Self {
        Tier {
            name: String::new(),
            size: 0,
            items: Vec::new(),
            interval_tier: true,
            tmin: 0.0,
            tmax: 0.0,
        }
    }

    pub fn add_empty_item(&mut self) {
        self.items.push(Item::new());
    }

    pub fn assert_valid(&self) -> Result<()> {
        if self.size != self.items.len() {
            return Err(data_error("Tier size does not match number of items"));
        }
        assert_valid_time_bounds(self.tmin, self.tmax, &format!("tier {}", self.name))?;
        for item_idx in 0..self.items.len() {
            let this_item = &self.items[item_idx];

            if self.interval_tier {
                assert_valid_time_bounds(
                    this_item.tmin,
                    this_item.tmax,
                    &format!("item {} in tier {}", item_idx, self.name),
                )?;
            } else {
                if (this_item.tmin - this_item.tmax).abs() > TIME_EPSILON {
                    return Err(data_error(&format!(
                        "Item {} should have tmin == tmax in PointTier {}",
                        item_idx, self.name
                    )));
                }
            }
            if item_idx + 1 < self.items.len() {
                let next_item = &self.items[item_idx + 1];
                if this_item.tmax - next_item.tmin > TIME_EPSILON {
                    return Err(data_error(&format!(
                        "Items {} and {} overlap in tier {}",
                        item_idx,
                        item_idx + 1,
                        self.name
                    )));
                }
            }
        }
        Ok(())
    }
}

impl TextGrid {
    pub fn new() -> Self {
        TextGrid {
            tmin: 0.0,
            tmax: 0.0,
            size: 0,
            name: String::new(),
            tiers: Vec::new(),
        }
    }

    pub fn add_empty_tier(&mut self) {
        self.tiers.push(Tier::new());
    }

    pub fn assert_valid(&self) -> Result<()> {
        if self.size != self.tiers.len() {
            return Err(data_error("TextGrid size does not match number of tiers"));
        }

        assert_valid_time_bounds(self.tmin, self.tmax, "TextGrid")?;

        for tier in &self.tiers {
            tier.assert_valid()?;
        }

        Ok(())
    }
}
