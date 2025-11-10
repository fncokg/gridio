use textgrid::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    static LONG_FILE: &str = "tests/data/long_format.TextGrid";
    static SHORT_FILE: &str = "tests/data/short_format.TextGrid";
    static NTIERS: usize = 5;

    fn assert_tgt_correct(tgt: &TextGrid) {
        assert_eq!(tgt.tiers.len(), NTIERS);
    }

    #[test]
    fn test_auto_read() {
        let tgt_long = read_from_file(LONG_FILE, true, "auto").unwrap();
        assert_tgt_correct(&tgt_long);
        let tgt_short = read_from_file(SHORT_FILE, true, "auto").unwrap();
        assert_tgt_correct(&tgt_short);
    }

    #[test]
    fn test_read_write_long() {
        let tgt = read_from_file_long(LONG_FILE, true).unwrap();
        tgt.save_textgrid("tmp_long.TextGrid", true);
        let org_content = fs::read_to_string(LONG_FILE).unwrap();
        let new_content = fs::read_to_string("tmp_long.TextGrid").unwrap();
        assert_eq!(org_content, new_content);

        let new_tgt = read_from_file_long("tmp_long.TextGrid", true).unwrap();
        assert_eq!(tgt.to_data(), new_tgt.to_data());
        fs::remove_file("tmp_long.TextGrid").unwrap();
    }
    #[test]
    fn test_read_write_short() {
        let tgt = read_from_file_short(SHORT_FILE, true).unwrap();
        tgt.save_textgrid("tmp_short.TextGrid", false);

        let org_content = fs::read_to_string(SHORT_FILE).unwrap();
        let new_content = fs::read_to_string("tmp_short.TextGrid").unwrap();
        assert_eq!(org_content, new_content);

        let new_tgt = read_from_file_short("tmp_short.TextGrid", true).unwrap();
        assert_eq!(tgt.to_data(), new_tgt.to_data());
        fs::remove_file("tmp_short.TextGrid").unwrap();
    }

    #[test]
    fn test_to_data_df() {
        let tgt = read_from_file(LONG_FILE, true, "long").unwrap();
        let (_, _, tiers) = tgt.to_data();
        let (tmins, tmaxs, labels, tier_names, is_intervals) = tgt.to_vectors();
        let nitems: usize = tiers.iter().map(|tier| tier.2.len()).sum();
        assert_eq!(tmins.len(), nitems);
        assert_eq!(tmaxs.len(), nitems);
        assert_eq!(labels.len(), nitems);
        assert_eq!(tier_names.len(), nitems);
        assert_eq!(is_intervals.len(), nitems);
        assert_eq!(tiers.len(), NTIERS);
    }

    #[test]
    fn test_from_data_df() {
        let tgt = read_from_file(LONG_FILE, true, "long").unwrap();
        let (tmin, tmax, tiers) = tgt.to_data();
        let (tmins, tmaxs, labels, tier_names, is_intervals) = tgt.to_vectors();

        let rebuilt_tgt_data =
            TextGrid::from_data(tiers, Some("rebuilt".to_string()), Some(tmin), Some(tmax))
                .unwrap();
        let rebuilt_tgt_vectors = TextGrid::from_vectors(
            tmins,
            tmaxs,
            labels,
            tier_names,
            is_intervals,
            None,
            None,
            Some("rebuilt2".to_string()),
        )
        .unwrap();
        assert_eq!(tgt.to_data(), rebuilt_tgt_data.to_data());
        assert_eq!(tgt.to_data(), rebuilt_tgt_vectors.to_data());
    }
}
