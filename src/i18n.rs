use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn from_code(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "en" => Some(Self::En),
            "zh" | "zh_cn" | "zh_tw" => Some(Self::Zh),
            _ => None,
        }
    }
}

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn init_from_env() {
    let lang = std::env::var("LANG")
        .ok()
        .and_then(|v| Lang::from_code(v.split(['.', '_']).next().unwrap_or("")))
        .unwrap_or(Lang::Zh);
    let _ = LANG.set(lang);
}

pub fn current() -> Lang {
    *LANG.get().unwrap_or(&Lang::Zh)
}

pub fn t(key: &str) -> &'static str {
    match current() {
        Lang::En => match key {
            "error" => "error",
            "warn" => "warn",
            "info" => "info",
            "done" => "done",
            _ => "unknown",
        },
        Lang::Zh => match key {
            "error" => "错误",
            "warn" => "警告",
            "info" => "提示",
            "done" => "完成",
            _ => "未知",
        },
    }
}
