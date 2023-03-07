use crate::io::files::create_leading_dirs;
use crate::net::download::DownloadError;
use crate::package::reg::PkgIdentifier;
use crate::io::files::paths::Paths;

use std::fmt::Display;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone)]
pub enum AssetKind {
	ResourcePack,
	Mod,
	Plugin,
	Shader
}

impl AssetKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"resource_pack" => Some(Self::ResourcePack),
			"mod" => Some(Self::Mod),
			"plugin" => Some(Self::Plugin),
			"shader" => Some(Self::Shader),
			_ => None
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Self::ResourcePack => String::from("resource_pack"),
			Self::Mod => String::from("mod"),
			Self::Plugin => String::from("plugin"),
			Self::Shader => String::from("shader")
		}
	}

	pub fn to_plural_string(&self) -> String {
		match self {
			Self::ResourcePack => String::from("resource_packs"),
			Self::Mod => String::from("mods"),
			Self::Plugin => String::from("plugins"),
			Self::Shader => String::from("shaders")
		}
	}
}

#[derive(Debug, Clone)]
pub struct Asset {
	pub kind: AssetKind,
	pub name: String,
	pub id: PkgIdentifier
}

impl Asset {
	pub fn new(kind: AssetKind, name: &str, id: PkgIdentifier) -> Self {
		Self {
			kind,
			name: name.to_owned(),
			id
		}
	}

	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		paths.mcvm_assets.join(self.kind.to_plural_string())
	}

	pub fn get_path(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths).join(&self.id.name).join(&self.id.version).join(&self.name)
	}
}

#[derive(Debug, Clone)]
pub struct AssetDownload {
	pub asset: Asset,
	url: String,
	force: bool
}

impl AssetDownload {
	pub fn new(asset: Asset, url: &str, force: bool) -> Self {
		Self {
			asset,
			url: url.to_owned(),
			force
		}
	}

	pub async fn download(&self, paths: &Paths) -> Result<(), DownloadError> {
		let path = self.asset.get_path(paths);
		if !self.force && path.exists() {
			return Ok(())
		}
		create_leading_dirs(&path)?;
		let client = reqwest::Client::new();
		let response = client.get(&self.url).send();
		fs::write(path, response.await?.bytes().await?)?;
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modloader {
	Vanilla,
	Forge,
	Fabric,
	Quilt
}

impl Modloader {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"forge" => Some(Self::Forge),
			"fabric" => Some(Self::Fabric),
			"quilt" => Some(Self::Quilt),
			_ => None
		}
	}
}

impl Display for Modloader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Vanilla => write!(f, "None"),
			Self::Forge => write!(f, "Forge"),
			Self::Fabric => write!(f, "Fabric"),
			Self::Quilt => write!(f, "Quilt")
		}
	}
}

#[derive(Debug, Clone)]
pub enum ModloaderMatch {
	Vanilla,
	Forge,
	Fabric,
	Quilt,
	FabricLike
}

impl ModloaderMatch {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"forge" => Some(Self::Forge),
			"fabric" => Some(Self::Fabric),
			"quilt" => Some(Self::Quilt),
			"fabriclike" => Some(Self::FabricLike),
			_ => None
		}
	}

	pub fn matches(&self, other: &Modloader) -> bool {
		match self {
			Self::Vanilla => matches!(other, Modloader::Vanilla),
			Self::Forge => matches!(other, Modloader::Forge),
			Self::Fabric => matches!(other, Modloader::Fabric),
			Self::Quilt => matches!(other, Modloader::Quilt),
			Self::FabricLike => matches!(other, Modloader::Fabric | Modloader::Quilt)
		}
	}
}

#[derive(Debug, Clone)]
pub enum PluginLoader {
	Vanilla,
	Paper
}

impl PluginLoader {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"paper" => Some(Self::Paper),
			_ => None
		}
	}
}

impl Display for PluginLoader {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Vanilla => write!(f, "None"),
			Self::Paper => write!(f, "Paper")
		}
	}
}

#[derive(Debug, Clone)]
pub enum PluginLoaderMatch {
	Vanilla,
	BukkitLike
}

impl PluginLoaderMatch {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"vanilla" => Some(Self::Vanilla),
			"bukkitlike" => Some(Self::BukkitLike),
			_ => None
		}
	}

	pub fn matches(&self, other: &PluginLoader) -> bool {
		match self {
			Self::Vanilla => matches!(other, PluginLoader::Vanilla),
			Self::BukkitLike => matches!(other, PluginLoader::Paper)
		}
	}
}

// Checks if the modloader and plugin loader are compatible with each other
pub fn game_modifications_compatible(modloader: &Modloader, plugin_loader: &PluginLoader) -> bool {
	match (modloader, plugin_loader) {
		(Modloader::Vanilla, _) => true,
		(_, PluginLoader::Vanilla) => true,
		_ => false
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_game_mods_compat() {
		assert!(game_modifications_compatible(&Modloader::Fabric, &PluginLoader::Vanilla));
		assert!(game_modifications_compatible(&Modloader::Vanilla, &PluginLoader::Vanilla));
		assert!(!game_modifications_compatible(&Modloader::Forge, &PluginLoader::Paper));
	}
}
