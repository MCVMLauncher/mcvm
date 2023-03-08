pub mod client;
pub mod server;

use color_print::cprintln;

use crate::io::java::{args::{MemoryNum, MemoryArg}, JavaKind};
use crate::io::files::paths::Paths;
use crate::util::json;
use crate::data::{instance::InstKind, user::Auth};

use super::{Instance, create::CreateError};

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
	#[error("Failed to create instance:\n{}", .0)]
	Create(#[from] CreateError),
	#[error("Java is not installed")]
	Java,
	#[error("Game process failed:\n{}", .0)]
	Command(std::io::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	Json(#[from] json::JsonError)
}

impl Instance {
	// Launch the instance
	pub async fn launch(
		&mut self,
		version_manifest: &json::JsonObject,
		paths: &Paths,
		auth: &Auth
	) -> Result<(), LaunchError> {
		cprintln!("Checking for updates...");
		match &self.kind {
			InstKind::Client => {
				self.create_client(version_manifest, paths, false, false).await?;
				cprintln!("<g>Launching!");
				self.launch_client(paths, auth)?;
			},
			InstKind::Server => {
				self.create_server(version_manifest, paths, false, false).await?;
				cprintln!("<g>Launching!");
				self.launch_server(paths)?;
			}
		}
		Ok(())
	}
}

#[derive(Debug)]
pub struct LaunchOptions {
	pub java: JavaKind,
	pub jvm_args: Vec<String>,
	pub game_args: Vec<String>,
	pub init_mem: Option<MemoryNum>,
	pub max_mem: Option<MemoryNum>,
}

impl LaunchOptions {
	pub fn generate_jvm_args(&self) -> Vec<String> {
		let mut out = self.jvm_args.clone();
		if let Some(n) = &self.init_mem {
			out.push(MemoryArg::Init.to_string(n.clone()));
		}
		if let Some(n) = &self.max_mem {
			out.push(MemoryArg::Max.to_string(n.clone()));
		}

		out
	}
}