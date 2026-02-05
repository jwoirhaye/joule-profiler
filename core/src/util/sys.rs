use std::{fs, sync::LazyLock};

static IS_ROOT: LazyLock<bool> = LazyLock::new(|| {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("Uid:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|uid| uid.parse::<u32>().ok())
        })
        == Some(0)
});

pub fn is_root() -> bool {
    *IS_ROOT
}
