use crate::textgrid::{Item, TextGrid, Tier};
use crate::utils::{fast_map, fast_move_map};
use std::io::{Error, ErrorKind, Result};

fn get_extreme<T, K>(items: &Vec<T>, key: K, find_max: bool) -> Option<f64>
where
    K: Fn(&T) -> f64,
{
    if items.is_empty() {
        return None;
    }
    let ext: f64;
    if find_max {
        ext = items
            .iter()
            .map(|item| key(item))
            .fold(f64::MIN, |a, b| a.max(b));
    } else {
        ext = items
            .iter()
            .map(|item| key(item))
            .fold(f64::MAX, |a, b| a.min(b));
    }
    Some(ext)
}

fn get_optional_extreme<T, F>(default: Option<f64>, items: &Vec<T>, key: F, find_max: bool) -> f64
where
    F: Fn(&T) -> f64,
{
    match default {
        Some(val) => val,
        None => get_extreme(items, key, find_max).unwrap_or(0.0),
    }
}

fn make_tier(
    items: Vec<Item>,
    tier_name: String,
    is_interval: bool,
    tmin: Option<f64>,
    tmax: Option<f64>,
) -> Tier {
    Tier {
        name: tier_name,
        interval_tier: is_interval,
        tmin: get_optional_extreme(tmin, &items, |item| item.tmin, false),
        tmax: get_optional_extreme(tmax, &items, |item| item.tmax, true),
        size: items.len(),
        items,
    }
}

fn make_textgrid(
    tiers: Vec<Tier>,
    name: Option<String>,
    tmin: Option<f64>,
    tmax: Option<f64>,
) -> Result<TextGrid> {
    let tgt = TextGrid {
        name: match name {
            Some(n) => n,
            None => String::from("ConvertedTextGrid"),
        },
        tmin: get_optional_extreme(tmin, &tiers, |tier| tier.tmin, false),
        tmax: get_optional_extreme(tmax, &tiers, |tier| tier.tmax, true),
        size: tiers.len(),
        tiers,
    };
    tgt.assert_valid()?;
    Ok(tgt)
}

impl TextGrid {
    pub fn to_data(&self) -> (f64, f64, Vec<(String, bool, Vec<(f64, f64, String)>)>) {
        let mut data = Vec::new();
        let map_fun = |item: &Item| (item.tmin, item.tmax, item.label.clone());
        for tier in self.tiers.iter() {
            let tier_data = (
                tier.name.clone(),
                tier.interval_tier,
                fast_map(&tier.items, map_fun, 20),
            );
            data.push(tier_data);
        }
        (self.tmin, self.tmax, data)
    }

    pub fn from_data(
        data: Vec<(String, bool, Vec<(f64, f64, String)>)>,
        name: Option<String>,
        tmin: Option<f64>,
        tmax: Option<f64>,
    ) -> Result<TextGrid> {
        let mut tiers = Vec::new();
        for (tier_name, is_interval, items_data) in data.into_iter() {
            let map_fun = |item_data: (f64, f64, String)| Item {
                tmin: item_data.0,
                tmax: item_data.1,
                label: item_data.2,
            };
            let items = fast_move_map(items_data, map_fun, 20);
            if items.is_empty() {
                continue;
            }
            let tier = make_tier(items, tier_name, is_interval, tmin, tmax);
            tiers.push(tier);
        }
        let tgt = make_textgrid(tiers, name, tmin, tmax)?;
        Ok(tgt)
    }

    pub fn to_vectors(&self) -> (Vec<f64>, Vec<f64>, Vec<String>, Vec<String>, Vec<bool>) {
        let mut tmins = Vec::new();
        let mut tmaxs = Vec::new();
        let mut labels = Vec::new();
        let mut tier_names = Vec::new();
        let mut is_intervals: Vec<bool> = Vec::new();
        for tier in self.tiers.iter() {
            for item in tier.items.iter() {
                tmins.push(item.tmin);
                tmaxs.push(item.tmax);
                labels.push(item.label.clone());
                is_intervals.push(tier.interval_tier);
                tier_names.push(tier.name.clone());
            }
        }
        (tmins, tmaxs, labels, tier_names, is_intervals)
    }

    pub fn from_vectors(
        tmins: Vec<f64>,
        tmaxs: Vec<f64>,
        labels: Vec<String>,
        tier_names: Vec<String>,
        is_intervals: Vec<bool>,
        tmin: Option<f64>,
        tmax: Option<f64>,
        name: Option<String>,
    ) -> Result<TextGrid> {
        if tmins.len() != tmaxs.len()
            || tmins.len() != labels.len()
            || tmins.len() != tier_names.len()
            || tmins.len() != is_intervals.len()
        {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Input vectors must have the same length",
            ));
        }
        let mut tier_map: std::collections::HashMap<String, (Vec<Item>, bool)> =
            std::collections::HashMap::new();

        // Also preserve the order of tier names, thus the order of the final TextGrid tiers is kept the same as the first appearance in vectors
        let mut tier_name_order: Vec<String> = Vec::new();
        for i in 0..tmins.len() {
            let item = Item {
                tmin: tmins[i],
                tmax: tmaxs[i],
                label: labels[i].clone(),
            };
            let tier_name: &String = &tier_names[i];
            if !tier_map.contains_key(tier_name) {
                tier_name_order.push(tier_name.clone());
                tier_map.insert(tier_name.clone(), (Vec::new(), is_intervals[i]));
            }
            tier_map.get_mut(tier_name).unwrap().0.push(item);
        }
        let mut tiers = Vec::new();
        for tier_name in tier_name_order.into_iter() {
            let (mut items, is_interval) = tier_map.remove(&tier_name).unwrap();
            // sort items by tmin
            items.sort_by(|a, b| a.tmin.partial_cmp(&b.tmin).unwrap());
            let tier = make_tier(items, tier_name, is_interval, tmin, tmax);
            tiers.push(tier);
        }
        let tgt = make_textgrid(tiers, name, tmin, tmax)?;
        Ok(tgt)
    }
}
