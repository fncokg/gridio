#[pyo3::pymodule]
mod pytextgrid {
    use numpy::{IntoPyArray, PyArray1};
    use pyo3::prelude::*;

    use textgrid::{files_to_data, files_to_vectors, read_from_file, TextGrid};

    #[pyfunction]
    pub fn textgrid2vectors<'py>(
        py: Python<'py>,
        file: &str,
        strict: bool,
        file_type: &str,
    ) -> PyResult<(
        Bound<'py, PyArray1<f32>>,
        Bound<'py, PyArray1<f32>>,
        Vec<String>,
        Vec<String>,
        Bound<'py, PyArray1<bool>>,
    )> {
        let tgt_result = read_from_file(file, strict, file_type);
        match tgt_result {
            Ok(tgt) => {
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
    #[pyfunction]
    pub fn textgrids2vectors<'py>(
        py: Python<'py>,
        files: Vec<String>,
        strict: bool,
        file_type: &str,
    ) -> PyResult<(
        Bound<'py, PyArray1<f32>>,
        Bound<'py, PyArray1<f32>>,
        Vec<String>,
        Vec<String>,
        Bound<'py, PyArray1<bool>>,
        Bound<'py, PyArray1<u32>>,
    )> {
        let vec_vectors = files_to_vectors(&files, strict, file_type);
        let mut tmins = Vec::new();
        let mut tmaxs = Vec::new();
        let mut labels = Vec::new();
        let mut tier_names = Vec::new();
        let mut is_intervals = Vec::new();
        let mut file_ids = Vec::new();
        for (i, (tmin_vec, tmax_vec, label_vec, tier_name_vec, is_interval_vec)) in
            vec_vectors.into_iter().enumerate()
        {
            file_ids.extend(vec![i as u32; tier_name_vec.len()]);
            tmins.extend(tmin_vec);
            tmaxs.extend(tmax_vec);
            labels.extend(label_vec);
            tier_names.extend(tier_name_vec);
            is_intervals.extend(is_interval_vec);
        }
        Ok((
            tmins.into_pyarray(py),
            tmaxs.into_pyarray(py),
            labels,
            tier_names,
            is_intervals.into_pyarray(py),
            file_ids.into_pyarray(py),
        ))
    }
    #[pyfunction]
    pub fn textgrid2data(
        file: &str,
        strict: bool,
        file_type: &str,
    ) -> PyResult<(f32, f32, Vec<(String, bool, Vec<(f32, f32, String)>)>)> {
        let tgt_result = read_from_file(file, strict, file_type);
        match tgt_result {
            Ok(tgt) => Ok(tgt.to_data()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to read TextGrid file {} because: {}",
                file, e
            ))),
        }
    }
    #[pyfunction]
    pub fn textgrids2data(
        files: Vec<String>,
        strict: bool,
        file_type: &str,
    ) -> PyResult<Vec<(f32, f32, Vec<(String, bool, Vec<(f32, f32, String)>)>)>> {
        let vec_data = files_to_data(&files, strict, file_type);
        Ok(vec_data)
    }
    #[pyfunction]
    pub fn data2textgrid(
        data: Vec<(String, bool, Vec<(f32, f32, String)>)>,
        tmin: Option<f32>,
        tmax: Option<f32>,
        output_file: &str,
        file_type: &str,
    ) -> PyResult<()> {
        let tgt_result = TextGrid::from_data(data, Some("TextGrid".to_string()), tmin, tmax);
        match tgt_result {
            Ok(tgt) => {
                tgt.save_textgrid(output_file, file_type == "long");
                Ok(())
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "Failed to create TextGrid because: {}",
                e
            ))),
        }
    }
    #[pyfunction]
    pub fn vectors2textgrid(
        tmins: Vec<f32>,
        tmaxs: Vec<f32>,
        labels: Vec<String>,
        tier_names: Vec<String>,
        is_intervals: Vec<bool>,
        tmin: Option<f32>,
        tmax: Option<f32>,
        output_file: &str,
        file_type: &str,
    ) -> PyResult<()> {
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
