use crate::textgrid::*;
use std::io::Result;

enum State {
    Header,
    Item,
    Tier,
    TierList,
}

fn parse_float(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

fn parse_str(s: &str) -> String {
    s.trim_matches('"').to_string()
}

fn parse_uint(s: &str) -> usize {
    s.parse().unwrap_or(0)
}

fn parse_item_kv(line: &str, item: &mut Item) {
    if let Some((key, value)) = parse_kv(line) {
        match key {
            "xmin" => item.tmin = parse_float(value),
            "xmax" => item.tmax = parse_float(value),
            "text" => item.label = parse_str(value),
            "number" => {
                item.tmin = parse_float(value);
                item.tmax = item.tmin;
            }
            "mark" => item.label = parse_str(value),
            _ => {}
        }
    }
}

fn parse_tier_kv(line: &str, tier: &mut Tier) {
    if let Some((key, value)) = parse_kv(line) {
        match key {
            "class" => match value.trim_matches('"') {
                "IntervalTier" => tier.interval_tier = true,
                "TextTier" => tier.interval_tier = false,
                _ => {
                    panic!("Unknown tier class: {}", value);
                }
            },
            "name" => tier.name = parse_str(value),
            "intervals: size" => tier.size = parse_uint(value),
            "points: size" => tier.size = parse_uint(value),
            "xmin" => tier.tmin = parse_float(value),
            "xmax" => tier.tmax = parse_float(value),
            _ => {}
        }
    }
}

fn parse_tg_kv(line: &str, tg: &mut TextGrid) {
    if let Some((key, value)) = parse_kv(line) {
        match key {
            "xmin" => tg.tmin = parse_float(value),
            "xmax" => tg.tmax = parse_float(value),
            "size" => tg.size = parse_uint(value),
            _ => {}
        }
    }
}

fn parse_kv(line: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), parts[1].trim()))
    } else {
        None
    }
}

pub fn read_from_file_long(fname: &str, strict: bool) -> Result<TextGrid> {
    let content = std::fs::read_to_string(fname).unwrap();
    let mut tg = TextGrid::new();
    tg.name = std::path::Path::new(fname)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut state = State::Header;
    for line in content.lines().map(|l| l.trim()) {
        if line.starts_with("item []") {
            state = State::TierList;
        } else if line.starts_with("item [") {
            state = State::Tier;
            tg.add_empty_tier();
        } else if line.starts_with("intervals [") || line.starts_with("points [") {
            state = State::Item;
            tg.tiers.last_mut().unwrap().add_empty_item();
        } else {
            // parse key-value pairs
            match state {
                State::Header => parse_tg_kv(line, &mut tg),
                State::Tier => parse_tier_kv(line, &mut tg.tiers.last_mut().unwrap()),
                State::Item => parse_item_kv(
                    line,
                    &mut tg.tiers.last_mut().unwrap().items.last_mut().unwrap(),
                ),
                // TierList has no key-value pairs
                State::TierList => (),
            }
        }
    }
    if strict {
        tg.assert_valid()?;
    }
    Ok(tg)
}
