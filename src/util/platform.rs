use anyhow::Result;

/// Checks if the operating system is Linux.
pub fn check_os() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        let os = std::env::consts::OS;
        Err(JouleProfilerError::UnsupportedOS(os.to_string()).into())
    }
}
