use std::path::PathBuf;

pub struct Config {
    pub knowledge_home: PathBuf,
    pub agents_home: PathBuf,
    pub dry_run: bool,
    pub auto_commit: bool,
}

impl Config {
    pub fn new(home_override: Option<PathBuf>, dry_run: bool) -> Self {
        let agents_home = home_override
            .clone()
            .map(|p| p.parent().unwrap_or(&p).to_path_buf())
            .or_else(|| {
                std::env::var("SAMSARA_HOME")
                    .ok()
                    .map(PathBuf::from)
                    .map(|p| p.parent().unwrap_or(&p).to_path_buf())
            })
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".agents")
            });

        let knowledge_home = home_override
            .or_else(|| std::env::var("SAMSARA_HOME").ok().map(PathBuf::from))
            .unwrap_or_else(|| agents_home.join("knowledge"));

        Config {
            knowledge_home,
            agents_home,
            dry_run,
            auto_commit: true,
        }
    }
}
