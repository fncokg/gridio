use crate::textgrid::{Item, TextGrid, Tier};
use crate::utils::{fast_enumerate_map, fast_map};

impl Tier {
    pub fn to_long_textgrid_string(&self, index: usize) -> String {
        let (tier_class, tier_name) = if self.interval_tier {
            ("IntervalTier", "intervals")
        } else {
            ("TextTier", "points")
        };
        let mut output = format!(
            "    item [{}]:\r\n        class = \"{}\" \r\n        name = \"{}\" \r\n        xmin = {} \r\n        xmax = {} \r\n        {}: size = {} \r\n",
            index + 1,
            tier_class,
            self.name.replace('"', "\\\""),
            self.tmin,
            self.tmax,
            tier_name,
            self.items.len()
        );
        let map_fun = |(index, item): (usize, &Item)| -> String {
            match tier_class {
                "IntervalTier" => {
                    format!(
                        "        intervals [{}]:\r\n            xmin = {} \r\n            xmax = {} \r\n            text = \"{}\" \r\n",
                        index + 1,
                        item.tmin,
                        item.tmax,
                        item.label.replace('"', "\\\"")
                    )
                }
                "TextTier" => {
                    format!(
                        "        points [{}]:\r\n            number = {} \r\n            mark = \"{}\" \r\n",
                        index + 1,
                        item.tmin,
                        item.label.replace('"', "\\\"")
                    )
                }
                _ => String::new(),
            }
        };
        let item_strings = fast_enumerate_map(&self.items, map_fun, 20);
        output.push_str(item_strings.join("").as_str());
        output
    }

    pub fn to_short_textgrid_string(&self) -> String {
        let tier_class;
        if self.interval_tier {
            tier_class = "IntervalTier";
        } else {
            tier_class = "TextTier";
        }
        let mut output = format!(
            "\"{}\"\r\n\"{}\"\r\n{}\r\n{}\r\n{}\r\n",
            tier_class,
            self.name.replace('"', "\\\""),
            self.tmin,
            self.tmax,
            self.items.len()
        );
        let map_fun = |item: &Item| -> String {
            match tier_class {
                "IntervalTier" => {
                    format!(
                        "{}\r\n{}\r\n\"{}\"\r\n",
                        item.tmin,
                        item.tmax,
                        item.label.replace('"', "\\\"")
                    )
                }
                "TextTier" => {
                    format!(
                        "{}\r\n\"{}\"\r\n",
                        item.tmin,
                        item.label.replace('"', "\\\"")
                    )
                }
                _ => String::new(),
            }
        };
        let item_strings = fast_map(&self.items, map_fun, 20);
        output.push_str(item_strings.join("").as_str());
        output
    }
}

impl TextGrid {
    pub fn to_long_textgrid_string(&self) -> String {
        // Note: In the long format, many lines are ended with a space character.
        // I don't know why and it seems unnecessary, but to be compatible, we add them here.
        let nitems = self.tiers.len();
        let tiers_existence = if nitems > 0 { "<exists>" } else { "<absent>" };
        let mut output = format!(
            "File type = \"ooTextFile\"\r\nObject class = \"TextGrid\"\r\n\r\nxmin = {} \r\nxmax = {} \r\ntiers? {} \r\nsize = {} \r\nitem []: \r\n",
            self.tmin, self.tmax, tiers_existence, nitems,
        );
        for (i, item) in self.tiers.iter().enumerate() {
            output.push_str(&item.to_long_textgrid_string(i));
        }
        output
    }

    pub fn to_short_textgrid_string(&self) -> String {
        let nitems = self.tiers.len();
        let tiers_existence = if nitems > 0 { "<exists>" } else { "<absent>" };
        let mut output = format!(
            "File type = \"ooTextFile\"\r\nObject class = \"TextGrid\"\r\n\r\n{}\r\n{}\r\n{}\r\n{}\r\n",
            self.tmin, self.tmax, tiers_existence, nitems,
        );
        for item in self.tiers.iter() {
            output.push_str(&item.to_short_textgrid_string());
        }
        output
    }

    pub fn save_textgrid(&self, filename: &str, long: bool) {
        let content = if long {
            self.to_long_textgrid_string()
        } else {
            self.to_short_textgrid_string()
        };
        std::fs::write(filename, content).unwrap();
    }

    pub fn save_csv(&self, filename: &str) {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(b',')
            .quote_style(csv::QuoteStyle::NonNumeric)
            .from_writer(std::fs::File::create(filename).unwrap());
        let (_, _, data) = self.to_data();
        wtr.write_record(&["tmin", "tmax", "label", "tier", "is_interval"])
            .unwrap();
        for (tier_name, is_interval, items) in data {
            for (tmin, tmax, label) in items {
                wtr.write_record(&[
                    tmin.to_string(),
                    tmax.to_string(),
                    label,
                    tier_name.clone(),
                    is_interval.to_string(),
                ])
                .unwrap();
            }
        }
    }
}
