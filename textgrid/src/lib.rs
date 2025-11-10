mod converter;
mod parser_long;
mod parser_short;
mod textgrid;
mod utils;
mod writer;

pub use parser_long::read_from_file_long;
pub use parser_short::read_from_file_short;
pub use textgrid::{Item, TextGrid, Tier};
pub use utils::{fast_map, fast_move_map};

use std::io::Result;

pub fn read_from_file(fname: &str, strict: bool, file_type: &str) -> Result<TextGrid> {
    match file_type {
        "long" => read_from_file_long(fname, strict),
        "short" => read_from_file_short(fname, strict),
        "auto" => {
            let content = std::fs::read_to_string(fname)?;
            if content.contains("item []") {
                read_from_file_long(fname, strict)
            } else {
                read_from_file_short(fname, strict)
            }
        }
        _ => panic!("Unknown file type: {}", file_type),
    }
}

pub fn files_to_data(
    fnames: &Vec<String>,
    strict: bool,
    file_type: &str,
) -> Vec<(f64, f64, Vec<(String, bool, Vec<(f64, f64, String)>)>)> {
    let map_fun = |tgt_fname: &String| {
        let tgt_result = read_from_file(tgt_fname, strict, file_type);
        match tgt_result {
            Ok(tgt) => tgt.to_data(),
            Err(_) => (0.0, 0.0, Vec::new()),
        }
    };
    let datas: Vec<(f64, f64, Vec<(String, bool, Vec<(f64, f64, String)>)>)> =
        fast_map(fnames, map_fun, 20);
    datas
}

pub fn files_to_vectors(
    fnames: &Vec<String>,
    strict: bool,
    file_type: &str,
) -> Vec<(Vec<f64>, Vec<f64>, Vec<String>, Vec<String>, Vec<bool>)> {
    let map_fun = |tgt_fname: &String| {
        let tgt_result = read_from_file(tgt_fname, strict, file_type);
        match tgt_result {
            Ok(tgt) => tgt.to_vectors(),
            Err(_) => (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
        }
    };
    let vectors: Vec<(Vec<f64>, Vec<f64>, Vec<String>, Vec<String>, Vec<bool>)> =
        fast_map(fnames, map_fun, 20);
    vectors
}
