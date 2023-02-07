#[derive(Debug, thiserror::Error)]
#[error("Version not found: {}", .version.as_string())]
pub struct VersionNotFoundError {
	pub version: MinecraftVersion
}

impl VersionNotFoundError {
	pub fn new(version: &MinecraftVersion) -> VersionNotFoundError {
		VersionNotFoundError{version: version.clone()}
	}
}

#[derive(Debug, Clone)]
pub enum MinecraftVersion {
	Unknown(String)
}

impl MinecraftVersion {
	pub fn from(string: &str) -> Self {
		Self::Unknown(string.to_string())
	}

	pub fn as_string(&self) -> &String {
		match self {
			Self::Unknown(string) => string
		}
	}
}

static _VERSION_LIST: [&str; 1] = ["1.19"];

pub enum VersionPattern {
	Single(String)
}

impl VersionPattern {
	pub fn matches(&self, versions: &Vec<String>) -> Option<String> {
		match self {
			VersionPattern::Single(version) => match versions.contains(version) {
				true => Some(version.to_string()),
				false => None
			}
		}
	}
}
