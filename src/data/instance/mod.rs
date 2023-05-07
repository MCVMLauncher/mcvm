pub mod create;
pub mod launch;

use anyhow::Context;
use mcvm_shared::instance::Side;

use crate::io::files::update_hardlink;
use crate::io::java::classpath::Classpath;
use crate::io::java::Java;
use crate::io::launch::LaunchOptions;
use crate::io::options::client::ClientOptions;
use crate::io::options::server::ServerOptions;
use crate::io::{files, Later};
use crate::net::fabric_quilt;
use crate::util::json;
use crate::Paths;

use super::addon::get_addon_path;
use super::config::instance::ClientWindowConfig;
use super::profile::update::UpdateManager;
use mcvm_shared::addon::{Addon, AddonKind};
use mcvm_shared::modifications::{Modloader, PluginLoader};

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum InstKind {
	Client {
		options: Option<Box<ClientOptions>>,
		window: ClientWindowConfig,
	},
	Server { options: Option<Box<ServerOptions>> },
}

impl InstKind {
	/// Convert to the Side enum
	pub fn to_side(&self) -> Side {
		match self {
			Self::Client { .. } => Side::Client,
			Self::Server { .. } => Side::Server,
		}
	}
}

#[derive(Debug)]
pub struct Instance {
	pub kind: InstKind,
	pub id: String,
	modloader: Modloader,
	plugin_loader: PluginLoader,
	launch: LaunchOptions,
	version_json: Later<Box<json::JsonObject>>,
	java: Later<Java>,
	classpath: Option<Classpath>,
	jar_path: Later<PathBuf>,
	main_class: Option<String>,
}

impl Instance {
	pub fn new(
		kind: InstKind,
		id: &str,
		modloader: Modloader,
		plugin_loader: PluginLoader,
		launch: LaunchOptions,
	) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			modloader,
			plugin_loader,
			launch,
			version_json: Later::new(),
			java: Later::new(),
			classpath: None,
			jar_path: Later::new(),
			main_class: None,
		}
	}

	pub fn get_dir(&self, paths: &Paths) -> PathBuf {
		match &self.kind {
			InstKind::Client { .. } => paths.project.data_dir().join("client").join(&self.id),
			InstKind::Server { .. } => paths.project.data_dir().join("server").join(&self.id),
		}
	}

	pub fn get_subdir(&self, paths: &Paths) -> PathBuf {
		self.get_dir(paths).join(match self.kind {
			InstKind::Client { .. } => ".minecraft",
			InstKind::Server { .. } => "server",
		})
	}

	/// Set the java installation for the instance
	fn add_java(&mut self, version: &str, manager: &UpdateManager) {
		let mut java = manager.java.get().clone();
		java.add_version(version);
		self.java.fill(java);
	}

	async fn get_fabric_quilt(
		&mut self,
		paths: &Paths,
		manager: &UpdateManager,
	) -> anyhow::Result<Classpath> {
		let meta = manager.fq_meta.get();
		let classpath = fabric_quilt::get_classpath(meta, paths, self.kind.to_side());
		self.main_class = Some(
			meta.launcher_meta
				.main_class
				.get_main_class_string(self.kind.to_side())
				.to_owned(),
		);

		Ok(classpath)
	}

	pub fn get_linked_addon_path(&self, addon: &Addon, paths: &Paths) -> Option<PathBuf> {
		let inst_dir = self.get_subdir(paths);
		match addon.kind {
			AddonKind::ResourcePack => {
				if let InstKind::Client { .. } = self.kind {
					Some(inst_dir.join("resourcepacks"))
				} else {
					None
				}
			}
			AddonKind::Mod => Some(inst_dir.join("mods")),
			AddonKind::Plugin => {
				if let InstKind::Server { .. } = self.kind {
					Some(inst_dir.join("plugins"))
				} else {
					None
				}
			}
			AddonKind::Shader => {
				if let InstKind::Client { .. } = self.kind {
					Some(inst_dir.join("shaders"))
				} else {
					None
				}
			}
		}
	}

	fn link_addon(dir: &Path, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		files::create_dir(dir)?;
		let link = dir.join(&addon.file_name);
		update_hardlink(&get_addon_path(addon, paths), &link)
			.context("Failed to create hard link")?;
		Ok(())
	}

	pub fn create_addon(&self, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		let inst_dir = self.get_subdir(paths);
		files::create_leading_dirs(&inst_dir)?;
		files::create_dir(&inst_dir)?;
		if let Some(path) = self.get_linked_addon_path(addon, paths) {
			Self::link_addon(&path, addon, paths)
				.with_context(|| format!("Failed to link addon {}", addon.id))?;
		}

		Ok(())
	}

	pub fn remove_addon(&self, addon: &Addon, paths: &Paths) -> anyhow::Result<()> {
		if let Some(path) = self.get_linked_addon_path(addon, paths) {
			let path = path.join(&addon.file_name);
			if path.exists() {
				fs::remove_file(&path)
					.with_context(|| format!("Failed to remove addon at {}", path.display()))?;
			}
		}

		Ok(())
	}

	// Removes the paper server jar file from a server instance
	pub fn remove_paper(&self, paths: &Paths, paper_file_name: String) -> anyhow::Result<()> {
		let inst_dir = self.get_subdir(paths);
		let paper_path = inst_dir.join(paper_file_name);
		if paper_path.exists() {
			fs::remove_file(paper_path).context("Failed to remove Paper jar")?;
		}

		Ok(())
	}

	// Removes files such as the game jar for when the profile version changes
	pub fn teardown(&self, paths: &Paths, paper_file_name: Option<String>) -> anyhow::Result<()> {
		match self.kind {
			InstKind::Client { .. } => {
				let inst_dir = self.get_dir(paths);
				let jar_path = inst_dir.join("client.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove client.jar")?;
				}
			}
			InstKind::Server { .. } => {
				let inst_dir = self.get_subdir(paths);
				let jar_path = inst_dir.join("server.jar");
				if jar_path.exists() {
					fs::remove_file(jar_path).context("Failed to remove server.jar")?;
				}

				if let Some(file_name) = paper_file_name {
					self.remove_paper(paths, file_name)
						.context("Failed to remove Paper")?;
				}
			}
		}

		Ok(())
	}
}
