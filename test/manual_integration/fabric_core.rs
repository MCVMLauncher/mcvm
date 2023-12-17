use std::path::PathBuf;

use anyhow::Context;
use mcvm_core::launch::LaunchConfiguration;
use mcvm_core::util::versions::MinecraftVersion;
use mcvm_core::{ClientWindowConfig, InstanceConfiguration, InstanceKind, MCVMCore};
use mcvm_mods::fabric_quilt;
use mcvm_shared::{output, Side};

#[tokio::main]
async fn main() {
	run().await.expect("Failed to run");
}

async fn run() -> anyhow::Result<()> {
	let version = "1.19.3";
	let mut o = output::Simple(output::MessageLevel::Trace);
	let mut core = MCVMCore::new().context("Failed to create core")?;
	let version_info = core
		.get_version_info(version.into())
		.await
		.context("Failed to get version info")?;

	let (classpath, main_class) = fabric_quilt::install_from_core(
		&mut core,
		&version_info,
		fabric_quilt::Mode::Fabric,
		Side::Client,
		&mut o,
	)
	.await
	.context("Failed to install Fabric/Quilt")?;

	let mut vers = core
		.get_version(&MinecraftVersion::Version(version.into()), &mut o)
		.await
		.context("Failed to create version")?;
	vers.ensure_client_assets_and_libs(&mut o)
		.await
		.context("Failed to ensure assets and libraries")?;

	let launch_config = LaunchConfiguration::new();
	let inst_config = InstanceConfiguration {
		side: InstanceKind::Client {
			window: ClientWindowConfig::new(),
		},
		path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("test/manual_integration/instances/fabric_core"),
		launch: launch_config,
		jar_path: None,
		main_class: Some(main_class),
		additional_libs: classpath.get_paths(),
	};
	let mut instance = vers
		.get_instance(inst_config, &mut o)
		.await
		.context("Failed to create instance")?;
	instance
		.launch(&mut o)
		.await
		.context("Failed to launch instance")?;

	Ok(())
}
