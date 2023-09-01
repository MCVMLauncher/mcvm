use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{data::profile::Profile, util::versions::MinecraftVersionDeser};

use mcvm_shared::{
	instance::Side,
	modifications::{ClientType, Modloader, ServerType},
	pkg::PackageStability,
};

use super::{instance::InstanceConfig, package::PackageConfig};

/// Game modifications
#[derive(Clone, Debug)]
pub struct GameModifications {
	modloader: Modloader,
	/// Type of the client
	pub client_type: ClientType,
	/// Type of the server
	pub server_type: ServerType,
}

impl GameModifications {
	/// Create a new GameModifications
	pub fn new(modloader: Modloader, client_type: ClientType, server_type: ServerType) -> Self {
		Self {
			modloader,
			client_type,
			server_type,
		}
	}

	/// Gets the modloader of a side
	pub fn get_modloader(&self, side: Side) -> Modloader {
		match side {
			Side::Client => match self.client_type {
				ClientType::None => self.modloader,
				ClientType::Vanilla => Modloader::Vanilla,
				ClientType::Forge => Modloader::Forge,
				ClientType::Fabric => Modloader::Fabric,
				ClientType::Quilt => Modloader::Quilt,
			},
			Side::Server => match self.server_type {
				ServerType::None => self.modloader,
				ServerType::Vanilla => Modloader::Vanilla,
				ServerType::Paper => Modloader::Vanilla,
				ServerType::Forge => Modloader::Forge,
				ServerType::Fabric => Modloader::Fabric,
				ServerType::Quilt => Modloader::Quilt,
			},
		}
	}

	/// Gets whether both client and server have the same modloader
	pub fn common_modloader(&self) -> bool {
		matches!(
			(self.client_type, self.server_type),
			(ClientType::None, ServerType::None)
				| (ClientType::Vanilla, ServerType::Vanilla)
				| (ClientType::Forge, ServerType::Forge)
				| (ClientType::Fabric, ServerType::Fabric)
				| (ClientType::Quilt, ServerType::Quilt)
		)
	}
}

/// Configuration for a profile
#[derive(Deserialize, Serialize)]
pub struct ProfileConfig {
	version: MinecraftVersionDeser,
	/// Configured modloader
	#[serde(default)]
	pub modloader: Modloader,
	/// Configured client type
	#[serde(default)]
	pub client_type: ClientType,
	/// Configured server type
	#[serde(default)]
	pub server_type: ServerType,
	/// Configured list of instances in this profile
	pub instances: HashMap<String, InstanceConfig>,
	/// Packages on this profile
	#[serde(default)]
	pub packages: Vec<PackageConfig>,
	/// Default stability setting of packages on this profile
	#[serde(default)]
	pub package_stability: PackageStability,
}

impl ProfileConfig {
	/// Creates a profile from this profile configuration
	pub fn to_profile(&self, profile_id: &str) -> Profile {
		Profile::new(
			profile_id,
			self.version.to_mc_version(),
			GameModifications::new(self.modloader, self.client_type, self.server_type),
		)
	}
}
