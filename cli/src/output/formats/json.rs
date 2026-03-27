use std::fs::File;
use std::io::Write;

use crate::output::displayer::error::IntoDisplayerError;
use crate::output::displayer::{Displayer, DisplayerError};
use joule_profiler_core::fs::{
    create_file_with_user_permissions, default_iterations_filename, get_absolute_path,
};
use joule_profiler_core::sensor::Sensor;
use joule_profiler_core::types::Iteration;
use serde_json::json;

type Result<T> = std::result::Result<T, DisplayerError>;

/// JSON output writer to a file.
pub struct JsonOutput {
    /// File writer.
    writer: File,

    /// Output filename.
    filename: String,
}

impl JsonOutput {
    /// Create a JSON output writer, optionally with a specific file.
    pub fn new(output_file: Option<String>) -> Result<Self> {
        let filename = output_file.unwrap_or(default_iterations_filename("json"));

        let absolute_path = get_absolute_path(&filename)?;
        let file = create_file_with_user_permissions(&absolute_path)?;

        Ok(Self {
            writer: file,
            filename: absolute_path,
        })
    }

    /// Write a JSON value to the output file.
    fn write_json(&mut self, value: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(value)
            .map_err(IntoDisplayerError::into_displayer_error)?;
        writeln!(self.writer, "{json_str}")?;
        println!("✔ JSON written to: {}", self.filename);
        Ok(())
    }
}

impl Displayer for JsonOutput {
    fn phases_single(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        result: &Iteration,
    ) -> Result<()> {
        let obj = json!({
            "command": cmd.join(" "),
            "mode": "phases",
            "token_pattern": token_pattern,
            "exit_code": result.exit_code,
            "phases": result.phases,
        });
        self.write_json(&obj)
    }

    fn phases_iterations(
        &mut self,
        cmd: &[String],
        token_pattern: &str,
        iterations: &[Iteration],
    ) -> Result<()> {
        let root = json!({
            "command": cmd.join(" "),
            "mode": "phases-iterations",
            "token_pattern": token_pattern,
            "nb_iterations": iterations.len(),
            "iterations": iterations
        });
        self.write_json(&root)
    }

    fn list_sensors(&mut self, sensors: &[Sensor]) -> Result<()> {
        self.write_json(
            &serde_json::to_value(sensors).map_err(IntoDisplayerError::into_displayer_error)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use joule_profiler_core::{
        sensor::Sensor,
        types::{Iteration, Metric, Phase, PhaseToken},
        unit::{MetricUnit, Unit, UnitPrefix},
    };
    use std::fs;
    use tempfile::NamedTempFile;

    fn unit() -> MetricUnit {
        MetricUnit {
            unit: Unit::Joule,
            prefix: UnitPrefix::Micro,
        }
    }

    fn metric(name: &str, value: u64) -> Metric {
        Metric {
            name: name.to_string(),
            value,
            unit: unit(),
            source: "rapl".to_string(),
        }
    }

    fn phase(start: PhaseToken, end: PhaseToken, metrics: Vec<Metric>) -> Phase {
        Phase {
            index: 0,
            start_token: start,
            end_token: end,
            duration_ms: 500,
            timestamp: 1000,
            start_token_line: None,
            end_token_line: None,
            metrics,
        }
    }

    fn iteration(index: usize, exit_code: i32, phases: Vec<Phase>) -> Iteration {
        Iteration {
            index,
            timestamp: 0,
            duration_ms: 0,
            exit_code,
            phases,
        }
    }

    fn json_to_tempfile() -> (JsonOutput, NamedTempFile) {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_owned();
        (JsonOutput::new(Some(path)).unwrap(), tmp)
    }

    fn read_json(tmp: &NamedTempFile) -> serde_json::Value {
        let content = fs::read_to_string(tmp.path()).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn new_valid_path_creates_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("out.json").to_str().unwrap().to_owned();
        assert!(JsonOutput::new(Some(path.clone())).is_ok());
        assert!(std::path::Path::new(&path).exists());
    }

    #[test]
    fn new_invalid_path_returns_error() {
        assert!(JsonOutput::new(Some("/nonexistent/dir/out.json".to_string())).is_err());
    }

    #[test]
    fn phases_single_root_fields() {
        let (mut json, tmp) = json_to_tempfile();
        let iter = iteration(
            0,
            42,
            vec![phase(
                PhaseToken::Start,
                PhaseToken::End,
                vec![metric("PKG", 10)],
            )],
        );
        json.phases_single(&["my_cmd".into(), "--flag".into()], "pattern", &iter)
            .unwrap();

        let v = read_json(&tmp);
        assert_eq!(v["command"], "my_cmd --flag");
        assert_eq!(v["mode"], "phases");
        assert_eq!(v["token_pattern"], "pattern");
        assert_eq!(v["exit_code"], 42);
    }

    #[test]
    fn phases_single_phases_array_length() {
        let (mut json, tmp) = json_to_tempfile();
        let iter = iteration(
            0,
            0,
            vec![
                phase(
                    PhaseToken::Start,
                    PhaseToken::Token("__A__".into()),
                    vec![metric("PKG", 1)],
                ),
                phase(
                    PhaseToken::Token("__A__".into()),
                    PhaseToken::End,
                    vec![metric("PKG", 2)],
                ),
            ],
        );
        json.phases_single(&["cmd".into()], ".*", &iter).unwrap();

        let v = read_json(&tmp);
        assert_eq!(v["phases"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn phases_single_metric_values_are_present() {
        let (mut json, tmp) = json_to_tempfile();
        let iter = iteration(
            0,
            0,
            vec![phase(
                PhaseToken::Start,
                PhaseToken::End,
                vec![metric("PKG", 999)],
            )],
        );
        json.phases_single(&["cmd".into()], ".*", &iter).unwrap();

        let v = read_json(&tmp);
        let metrics = &v["phases"][0]["metrics"];
        assert_eq!(metrics[0]["name"], "PKG");
        assert_eq!(metrics[0]["value"], 999);
        assert_eq!(metrics[0]["source"], "rapl");
    }

    #[test]
    fn phases_iterations_root_fields() {
        let (mut json, tmp) = json_to_tempfile();
        let iters = vec![
            iteration(
                0,
                0,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 1)],
                )],
            ),
            iteration(
                1,
                0,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 2)],
                )],
            ),
        ];
        json.phases_iterations(&["cmd".into()], "PAT", &iters)
            .unwrap();

        let v = read_json(&tmp);
        assert_eq!(v["mode"], "phases-iterations");
        assert_eq!(v["token_pattern"], "PAT");
        assert_eq!(v["nb_iterations"], 2);
        assert_eq!(v["command"], "cmd");
    }

    #[test]
    fn phases_iterations_iterations_array_length() {
        let (mut json, tmp) = json_to_tempfile();
        let iters = vec![
            iteration(
                0,
                0,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 1)],
                )],
            ),
            iteration(
                1,
                0,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 2)],
                )],
            ),
            iteration(
                2,
                0,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 3)],
                )],
            ),
        ];
        json.phases_iterations(&["cmd".into()], ".*", &iters)
            .unwrap();

        let v = read_json(&tmp);
        assert_eq!(v["iterations"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn phases_iterations_exit_codes_per_iteration() {
        let (mut json, tmp) = json_to_tempfile();
        let iters = vec![
            iteration(
                0,
                0,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 1)],
                )],
            ),
            iteration(
                1,
                42,
                vec![phase(
                    PhaseToken::Start,
                    PhaseToken::End,
                    vec![metric("PKG", 2)],
                )],
            ),
        ];
        json.phases_iterations(&["cmd".into()], ".*", &iters)
            .unwrap();

        let v = read_json(&tmp);
        assert_eq!(v["iterations"][0]["exit_code"], 0);
        assert_eq!(v["iterations"][1]["exit_code"], 42);
    }

    #[test]
    fn list_sensors_writes_array() {
        let (mut json, tmp) = json_to_tempfile();
        let sensors = vec![
            Sensor {
                name: "PKG".into(),
                unit: unit(),
                source: "rapl".into(),
            },
            Sensor {
                name: "DRAM".into(),
                unit: unit(),
                source: "rapl".into(),
            },
        ];
        json.list_sensors(&sensors).unwrap();

        let v = read_json(&tmp);
        assert!(v.is_array());
        assert_eq!(v.as_array().unwrap().len(), 2);
        assert_eq!(v[0]["name"], "PKG");
        assert_eq!(v[1]["name"], "DRAM");
    }

    #[test]
    fn list_sensors_empty_writes_empty_array() {
        let (mut json, tmp) = json_to_tempfile();
        json.list_sensors(&[]).unwrap();
        let v = read_json(&tmp);
        assert!(v.is_array());
        assert!(v.as_array().unwrap().is_empty());
    }
}
