/// Python module for TextGrid file manipulation.
///
/// This module provides a high-performance Python interface for reading, writing,
/// and converting Praat TextGrid files using Rust bindings. It supports both
/// long and short TextGrid formats and offers efficient batch processing capabilities.
#[pyo3::pymodule]
mod pytextgrid {
    use numpy::{IntoPyArray, PyArray1};
    use pyo3::prelude::*;

    use textgrid::{files_to_data, files_to_vectors, read_from_file, TextGrid};

    /// Converts a single TextGrid file to vectorized format.
    ///
    /// This function reads a TextGrid file and converts it into numpy arrays and vectors,
    /// which is optimized for data analysis and machine learning workflows.
    ///
    /// # Arguments
    ///
    /// * `py` - Python interpreter token for creating numpy arrays
    /// * `file` - Path to the TextGrid file to read
    /// * `strict` - If true, enforces strict parsing rules; if false, allows more lenient parsing
    /// * `file_type` - Format type of the file: "long" or "short"
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `PyArray1<f64>` - Start times (tmin) of all intervals/points, moved as `numpy.ndarray`
    /// * `PyArray1<f64>` - End times (tmax) of all intervals/points, moved as `numpy.ndarray`
    /// * `Vec<String>` - Labels/text of all intervals/points, copied as Python `list`
    /// * `Vec<String>` - Tier names for each interval/point, copied as Python `list`
    /// * `PyArray1<bool>` - Boolean flags indicating if each entry is an interval (true) or point (false), moved as `numpy.ndarray`
    ///
    /// # Errors
    ///
    /// Returns a PyIOError if the file cannot be read or parsed.
    #[pyfunction]
    pub fn textgrid2vectors<'py>(
        py: Python<'py>,
        file: &str,
        strict: bool,
        file_type: &str,
    ) -> PyResult<(
        Bound<'py, PyArray1<f64>>,
        Bound<'py, PyArray1<f64>>,
        Vec<String>,
        Vec<String>,
        Bound<'py, PyArray1<bool>>,
    )> {
        // Parse the TextGrid file into Rust structure
        let tgt_result = read_from_file(file, strict, file_type);
        match tgt_result {
            Ok(tgt) => {
                // Convert TextGrid structure to vectors
                let (tmins, tmaxs, labels, tier_names, is_intervals) = tgt.to_vectors();
                Ok((
                    tmins.into_pyarray(py),
                    tmaxs.into_pyarray(py),
                    labels,
                    tier_names,
                    is_intervals.into_pyarray(py),
                ))
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to read TextGrid file {} because: {}",
                file, e
            ))),
        }
    }

    /// Converts multiple TextGrid files to vectorized format with file tracking.
    ///
    /// This function batch-processes multiple TextGrid files and converts them into
    /// a single unified set of vectors, with an additional file ID array to track
    /// which file each entry came from. This is useful for bulk analysis and
    /// comparing multiple recordings.
    ///
    /// # Arguments
    ///
    /// * `py` - Python interpreter token for creating numpy arrays
    /// * `files` - Vector of file paths to process
    /// * `strict` - If true, enforces strict parsing rules; if false, allows more lenient parsing
    /// * `file_type` - Format type: "long" or "short"
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `PyArray1<f64>` - Concatenated start times from all files, moved as `numpy.ndarray`
    /// * `PyArray1<f64>` - Concatenated end times from all files, moved as `numpy.ndarray`
    /// * `Vec<String>` - Concatenated labels from all files, copied as Python `list`
    /// * `Vec<String>` - Concatenated tier names from all files, copied as Python `list`
    /// * `PyArray1<bool>` - Concatenated interval/point flags from all files, moved as `numpy.ndarray`
    /// * `PyArray1<u32>` - File ID for each entry (0-indexed), moved as `numpy.ndarray`
    ///
    /// # Errors
    ///
    /// Returns a PyIOError if any file cannot be read or parsed.
    #[pyfunction]
    pub fn textgrids2vectors<'py>(
        py: Python<'py>,
        files: Vec<String>,
        strict: bool,
        file_type: &str,
    ) -> PyResult<(
        Bound<'py, PyArray1<f64>>,
        Bound<'py, PyArray1<f64>>,
        Vec<String>,
        Vec<String>,
        Bound<'py, PyArray1<bool>>,
        Bound<'py, PyArray1<u32>>,
    )> {
        // Process all files and get their individual vector representations
        let vec_vectors = files_to_vectors(&files, strict, file_type);

        // Initialize vectors to hold concatenated results from all files
        let mut tmins = Vec::new();
        let mut tmaxs = Vec::new();
        let mut labels = Vec::new();
        let mut tier_names = Vec::new();
        let mut is_intervals = Vec::new();
        let mut file_ids = Vec::new();

        // Iterate through each file's vectors and concatenate them
        // Note: enumerate() provides file index for tracking source file
        for (i, (tmin_vec, tmax_vec, label_vec, tier_name_vec, is_interval_vec)) in
            vec_vectors.into_iter().enumerate()
        {
            file_ids.extend(vec![i as u32; tier_name_vec.len()]);

            // Concatenate all data vectors
            tmins.extend(tmin_vec);
            tmaxs.extend(tmax_vec);
            labels.extend(label_vec);
            tier_names.extend(tier_name_vec);
            is_intervals.extend(is_interval_vec);
        }

        Ok((
            tmins.into_pyarray(py),        // Vec<f64> -> numpy.ndarray
            tmaxs.into_pyarray(py),        // Vec<f64> -> numpy.ndarray
            labels,                        // Vec<String> -> Python list
            tier_names,                    // Vec<String> -> Python list
            is_intervals.into_pyarray(py), // Vec<bool> -> numpy.ndarray
            file_ids.into_pyarray(py),     // Vec<u32> -> numpy.ndarray
        ))
    }

    /// Converts a single TextGrid file to structured data format.
    ///
    /// This function reads a TextGrid file and returns it as nested tuples/lists,
    /// preserving the hierarchical structure of the TextGrid (tiers and their items).
    /// This format is more intuitive for manual inspection and manipulation.
    ///
    /// # Arguments
    ///
    /// * `file` - Path to the TextGrid file
    /// * `strict` - If true, enforces strict parsing; if false, allows lenient parsing
    /// * `file_type` - Format type: "long" or "short"
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * `f64` - Global start time (tmin) of the TextGrid
    /// * `f64` - Global end time (tmax) of the TextGrid
    /// * `Vec<(String, bool, Vec<(f64, f64, String)>)>` - Vector of tiers, where each tier is:
    ///   - `String`: Tier name
    ///   - `bool`: True if interval tier, false if point tier
    ///   - `Vec<(f64, f64, String)>`: Vector of items (tmin, tmax, label)
    ///
    /// # Errors
    ///
    /// Returns a PyIOError if the file cannot be read or parsed.
    #[pyfunction]
    pub fn textgrid2data(
        file: &str,
        strict: bool,
        file_type: &str,
    ) -> PyResult<(f64, f64, Vec<(String, bool, Vec<(f64, f64, String)>)>)> {
        // Parse TextGrid file
        let tgt_result = read_from_file(file, strict, file_type);
        match tgt_result {
            // Type note: Rust nested tuples/vectors are automatically converted
            // to Python nested tuples/lists by PyO3
            Ok(tgt) => Ok(tgt.to_data()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to read TextGrid file {} because: {}",
                file, e
            ))),
        }
    }

    /// Converts multiple TextGrid files to structured data format.
    ///
    /// This function batch-processes multiple TextGrid files and returns them as
    /// a vector of structured data, where each element represents one file's content.
    ///
    /// # Arguments
    ///
    /// * `files` - Vector of file paths to process
    /// * `strict` - If true, enforces strict parsing; if false, allows lenient parsing
    /// * `file_type` - Format type: "long" or "short"
    ///
    /// # Returns
    ///
    /// A vector where each element is a tuple for one file:
    /// * `f64` - Global start time (tmin) of the TextGrid
    /// * `f64` - Global end time (tmax) of the TextGrid
    /// * `Vec<(String, bool, Vec<(f64, f64, String)>)>` - Tiers data (name, is_interval, items)
    ///
    /// # Errors
    ///
    /// Returns a PyIOError if any file cannot be read or parsed.
    #[pyfunction]
    pub fn textgrids2data(
        files: Vec<String>,
        strict: bool,
        file_type: &str,
    ) -> PyResult<Vec<(f64, f64, Vec<(String, bool, Vec<(f64, f64, String)>)>)>> {
        // Process all files in batch
        let vec_data = files_to_data(&files, strict, file_type);
        // Type note: Rust Vec<nested_tuple> -> Python list of nested tuples
        Ok(vec_data)
    }

    /// Creates a TextGrid file from structured data format.
    ///
    /// This function takes hierarchical tier data and writes it to a TextGrid file
    /// in either long or short format. The global time bounds can be specified or
    /// will be automatically calculated from the data.
    ///
    /// # Arguments
    ///
    /// * `data` - Structured tier data: Vec of (tier_name, is_interval, items)
    ///   where items is Vec of (tmin, tmax, label)
    /// * `tmin` - Optional global start time. If None, calculated from data
    /// * `tmax` - Optional global end time. If None, calculated from data
    /// * `output_file` - Path where the TextGrid file will be saved
    /// * `file_type` - Output format: "long" for long format, anything else for short format
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success.
    ///
    /// # Errors
    ///
    /// Returns a PyIOError if the TextGrid cannot be created or saved.
    ///
    /// # Type Conversions
    ///
    /// Python inputs are automatically converted:
    /// * Python list -> Rust Vec
    /// * Python tuple -> Rust tuple
    /// * Python str -> Rust String/&str
    /// * Python float -> Rust f64
    /// * Python bool -> Rust bool
    /// * Python None -> Rust Option::None
    #[pyfunction]
    pub fn data2textgrid(
        data: Vec<(String, bool, Vec<(f64, f64, String)>)>,
        tmin: Option<f64>,
        tmax: Option<f64>,
        output_file: &str,
        file_type: &str,
    ) -> PyResult<()> {
        // Create TextGrid structure from data
        // Type conversion: Python nested structures -> Rust nested structures (automatic via PyO3)
        let tgt_result = TextGrid::from_data(data, Some("TextGrid".to_string()), tmin, tmax);
        match tgt_result {
            Ok(tgt) => {
                // Write to file: true for long format, false for short format
                tgt.save_textgrid(output_file, file_type == "long");
                Ok(())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create TextGrid because: {}",
                e
            ))),
        }
    }

    /// Creates a TextGrid file from vectorized format.
    ///
    /// This function takes parallel vectors of data and reconstructs a TextGrid file.
    /// This is the inverse operation of `textgrid2vectors` and is useful when working
    /// with data analysis pipelines that operate on vectors/arrays.
    ///
    /// # Arguments
    ///
    /// * `tmins` - Start times for all intervals/points
    /// * `tmaxs` - End times for all intervals/points
    /// * `labels` - Text labels for all intervals/points
    /// * `tier_names` - Tier name for each interval/point (determines tier grouping)
    /// * `is_intervals` - Boolean flags: true for interval, false for point
    /// * `tmin` - Optional global start time. If None, calculated from data
    /// * `tmax` - Optional global end time. If None, calculated from data
    /// * `output_file` - Path where the TextGrid file will be saved
    /// * `file_type` - Output format: "long" for long format, anything else for short format
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success.
    ///
    /// # Errors
    ///
    /// Returns a PyIOError if the vectors are inconsistent or the file cannot be saved.
    ///
    /// # Type Conversions
    ///
    /// Python inputs are converted to Rust types:
    /// * Python list[float] -> Rust Vec<f64>
    /// * Python list[str] -> Rust Vec<String>
    /// * Python list[bool] -> Rust Vec<bool>
    /// * Python None -> Rust Option::None
    /// * Python float -> Rust f64 (for tmin/tmax)
    ///
    /// # Notes
    ///
    /// All vectors must have the same length. Items with the same tier_name
    /// will be grouped into the same tier in the output file.
    #[pyfunction]
    pub fn vectors2textgrid(
        tmins: Vec<f64>,
        tmaxs: Vec<f64>,
        labels: Vec<String>,
        tier_names: Vec<String>,
        is_intervals: Vec<bool>,
        tmin: Option<f64>,
        tmax: Option<f64>,
        output_file: &str,
        file_type: &str,
    ) -> PyResult<()> {
        // Create TextGrid from parallel vectors
        // Type conversion: Python lists -> Rust Vec (automatic via PyO3)
        // Important: All vectors must have the same length for proper alignment
        let tgt_result = TextGrid::from_vectors(
            tmins,
            tmaxs,
            labels,
            tier_names,
            is_intervals,
            tmin,
            tmax,
            Some("TextGrid".to_string()),
        );
        match tgt_result {
            Ok(tgt) => {
                // Write to file: true for long format, false for short format
                tgt.save_textgrid(output_file, file_type == "long");
                Ok(())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create TextGrid because: {}",
                e
            ))),
        }
    }
}
