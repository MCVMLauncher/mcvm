pub mod update;

use anyhow::Context;

use crate::data::instance::Instance;
use crate::package::PkgProfileConfig;
use crate::util::versions::MinecraftVersion;
use crate::Paths;

use self::update::UpdateManager;

use super::addon::Modloader;
use super::addon::PluginLoader;

pub type InstanceRegistry = std::collections::HashMap<String, Instance>;

#[derive(Debug)]
pub struct Profile {
	pub name: String,
	pub version: MinecraftVersion,
	pub instances: Vec<String>,
	pub packages: Vec<PkgProfileConfig>,
	pub modloader: Modloader,
	pub plugin_loader: PluginLoader,
}

impl Profile {
	pub fn new(
		name: &str,
		version: MinecraftVersion,
		modloader: Modloader,
		plugin_loader: PluginLoader,
	) -> Self {
		Profile {
			name: name.to_owned(),
			version: version.to_owned(),
			instances: Vec::new(),
			packages: Vec::new(),
			modloader,
			plugin_loader,
		}
	}

	pub fn add_instance(&mut self, instance: &str) {
		self.instances.push(instance.to_owned());
	}

	/// Create all the instances in this profile. Returns the version list.
	pub async fn create_instances(
		&mut self,
		reg: &mut InstanceRegistry,
		paths: &Paths,
		mut manager: UpdateManager,
	) -> anyhow::Result<Vec<String>> {
		for id in self.instances.iter_mut() {
			let instance = reg.get(id).expect("Profile has unknown instance");
			manager.add_requirements(instance.get_requirements());
		}
		manager.fulfill_requirements(paths).await?;
		for id in self.instances.iter_mut() {
			let instance = reg.get_mut(id).expect("Profile has unknown instance");
			let files = instance
				.create(&manager, paths)
				.await
				.with_context(|| format!("Failed to create instance {id}"))?;
			manager.add_files(files);
		}
		Ok(manager.version_list.get_val())
	}
}
