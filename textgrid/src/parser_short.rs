use crate::textgrid::*;
use std::io::Result;

fn parse_float(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}

fn parse_str(s: &str) -> String {
    s.trim_matches('"').to_string()
}

fn parse_uint(s: &str) -> usize {
    s.parse().unwrap_or(0)
}

fn parse_tier(lines: &[&str], start_index: usize) -> (Tier, usize) {
    let mut tier = Tier::new();
    tier.interval_tier = match lines[start_index].trim_matches('"') {
        "IntervalTier" => true,
        "TextTier" => false,
        _ => {
            panic!("Unknown tier class: {}", lines[start_index]);
        }
    };
    tier.name = parse_str(lines[start_index + 1]);
    tier.tmin = parse_float(lines[start_index + 2]);
    tier.tmax = parse_float(lines[start_index + 3]);
    tier.size = parse_uint(lines[start_index + 4]);
    let mut cursor = start_index + 5;
    for _ in 0..tier.size {
        let item: Item;
        if tier.interval_tier {
            item = Item {
                tmin: parse_float(lines[cursor]),
                tmax: parse_float(lines[cursor + 1]),
                label: parse_str(lines[cursor + 2]),
            };
            cursor += 3;
        } else {
            let number = parse_float(lines[cursor]);
            item = Item {
                tmin: number,
                tmax: number,
                label: parse_str(lines[cursor + 1]),
            };
            cursor += 2;
        }
        tier.items.push(item);
    }
    (tier, cursor)
}

pub fn read_from_file_short(fname: &str, strict: bool) -> Result<TextGrid> {
    let content = std::fs::read_to_string(fname).unwrap();
    let mut tg = TextGrid::new();
    tg.name = std::path::Path::new(fname)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let lines: Vec<&str> = content.lines().map(|l| l.trim()).collect();

    tg.tmin = parse_float(lines[3]);
    tg.tmax = parse_float(lines[4]);
    tg.size = parse_uint(lines[6]);

    let mut cursor = 7;
    for _ in 0..tg.size {
        let (tier, next_cursor) = parse_tier(&lines, cursor);
        tg.tiers.push(tier);
        cursor = next_cursor;
    }

    if strict {
        tg.assert_valid()?;
    }
    Ok(tg)
}
