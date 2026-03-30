use joule_profiler_core::{
    JouleProfiler, config::ProfileConfig, sensor::Sensors, source::MetricReader, types::{Metrics, PhaseToken}
};
use mockall::mock;

#[derive(Debug)]
pub struct MockError;

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mock error")
    }
}
impl std::error::Error for MockError {}

mock! {
    pub MetricReader {}

    impl MetricReader for MetricReader {
        type Type = ();
        type Error = MockError;

        async fn init(&mut self, pid: i32) -> Result<(), MockError>;
        async fn join(&mut self) -> Result<(), MockError>;
        async fn measure(&mut self) -> Result<(), MockError>;
        async fn reset(&mut self) -> Result<(), MockError>;
        async fn retrieve(&mut self) -> Result<(), MockError>;
        fn get_sensors(&self) -> Result<Sensors, MockError>;
        fn to_metrics(&self, v: ()) -> Result<Metrics, MockError>;
        fn get_name() -> &'static str;
    }
}

fn mock_reader() -> MockMetricReader {
    let mut mock = MockMetricReader::new();
    mock.expect_init().returning(|_| Ok(()));
    mock.expect_join().returning(||  Ok(()));
    mock.expect_measure().returning(||  Ok(()));
    mock.expect_reset().returning(||  Ok(()));
    mock.expect_retrieve().returning(||  Ok(()));
    mock.expect_get_sensors().returning(||  Ok(Sensors::default()));
    mock.expect_to_metrics().returning(|_| Ok(Metrics::default()));
    mock
}

fn config(cmd: Vec<String>, pattern: &str) -> ProfileConfig {
    ProfileConfig {
        cmd,
        iterations: 1,
        token_pattern: pattern.to_string(),
        stdout_file: None,
    }
}

#[tokio::test]
async fn profile_no_phases() {
    let mut profiler = JouleProfiler::new();
    profiler.add_source(mock_reader());
    let config = config(vec!["echo".into(), "hello world".into()], "__PHASE__");
    let result = profiler.profile(&config).await.unwrap();
    assert_eq!(result.len(), 1);
    let iteration = &result[0];
    assert_eq!(iteration.exit_code, 0);
    assert_eq!(iteration.phases.len(), 1);
    assert_eq!(iteration.phases[0].start_token, PhaseToken::Start);
    assert_eq!(iteration.phases[0].end_token, PhaseToken::End);
}

#[tokio::test]
async fn profile_single_phase_token() {
    let mut profiler = JouleProfiler::new();
    profiler.add_source(mock_reader());
    let config = config(
        vec!["echo".into(), "__PHASE_1__".into()],
        "__PHASE_[0-9]+__",
    );
    let result = profiler.profile(&config).await.unwrap();
    let iteration = &result[0];
    assert_eq!(iteration.exit_code, 0);
    assert_eq!(iteration.phases.len(), 2);
    assert_eq!(iteration.phases[0].end_token,   PhaseToken::Token("__PHASE_1__".into()));
    assert_eq!(iteration.phases[1].start_token, PhaseToken::Token("__PHASE_1__".into()));
    assert_eq!(iteration.phases[1].end_token,   PhaseToken::End);
}

#[tokio::test]
async fn profile_multiple_phase_tokens() {
    let mut profiler = JouleProfiler::new();
    profiler.add_source(mock_reader());
    let config = config(
        vec![
            "sh".into(),
            "-c".into(),
            "printf '__PHASE_1__\\n__PHASE_2__\\n__PHASE_3__\\n'".into(),
        ],
        "__PHASE_[0-9]+__",
    );
    let result = profiler.profile(&config).await.unwrap();
    let phases = &result[0].phases;
    assert_eq!(phases.len(), 4);
    assert_eq!(phases[0].start_token, PhaseToken::Start);
    assert_eq!(phases[0].end_token,   PhaseToken::Token("__PHASE_1__".into()));
    assert_eq!(phases[1].start_token, PhaseToken::Token("__PHASE_1__".into()));
    assert_eq!(phases[1].end_token,   PhaseToken::Token("__PHASE_2__".into()));
    assert_eq!(phases[2].end_token,   PhaseToken::Token("__PHASE_3__".into()));
    assert_eq!(phases[3].start_token, PhaseToken::Token("__PHASE_3__".into()));
    assert_eq!(phases[3].end_token,   PhaseToken::End);
}

#[tokio::test]
async fn profile_multiple_iterations() {
    let mut profiler = JouleProfiler::new();
    profiler.add_source(mock_reader());
    let mut config = config(
        vec!["echo".into(), "__PHASE_1__".into()],
        "__PHASE_[0-9]+__",
    );
    config.iterations = 3;
    let result = profiler.profile(&config).await.unwrap();
    assert_eq!(result.len(), 3);
    for (i, iteration) in result.iter().enumerate() {
        assert_eq!(iteration.index, i);
        assert_eq!(iteration.exit_code, 0);
        assert_eq!(iteration.phases.len(), 2);
    }
}

#[tokio::test]
async fn profile_nonzero_exit_code_is_reported() {
    let mut profiler = JouleProfiler::new();
    profiler.add_source(mock_reader());
    let config = config(
        vec!["sh".into(), "-c".into(), "exit 42".into()],
        "__PHASE__",
    );
    let result = profiler.profile(&config).await.unwrap();
    assert_eq!(result[0].exit_code, 42);
}