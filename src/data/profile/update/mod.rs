/// UpdateManager
pub mod manager;
/// Updating packages on a profile
pub mod packages;

use std::collections::HashSet;

use anyhow::{anyhow, Context};
use mcvm_shared::modifications::ServerType;
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use reqwest::Client;

use crate::data::config::Config;
use crate::io::files::paths::Paths;
use crate::io::lock::Lockfile;
use crate::net::paper;
use crate::package::eval::EvalConstants;
use crate::package::reg::PkgRegistry;
use crate::util::print::PrintOptions;

use manager::UpdateManager;
use packages::{print_package_support_messages, update_profile_packages};

use super::{InstanceRegistry, Profile};

/// Shared objects for profile updating functions
pub struct ProfileUpdateContext<'a, O: MCVMOutput> {
	pub packages: &'a mut PkgRegistry,
	pub instances: &'a mut InstanceRegistry,
	pub paths: &'a Paths,
	pub lock: &'a mut Lockfile,
	pub client: &'a Client,
	pub output: &'a mut O,
}

/// Update a list of profiles
pub async fn update_profiles(
	paths: &Paths,
	config: &mut Config,
	ids: &[String],
	force: bool,
	update_packages: bool,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<()> {
	let mut all_packages = HashSet::new();
	let client = Client::new();
	let mut lock = Lockfile::open(paths).context("Failed to open lockfile")?;

	let mut ctx = ProfileUpdateContext {
		packages: &mut config.packages,
		instances: &mut config.instances,
		paths,
		lock: &mut lock,
		client: &client,
		output: o,
	};

	for id in ids {
		let profile = config
			.profiles
			.get_mut(id)
			.ok_or(anyhow!("Unknown profile '{id}'"))?;

		ctx.output.display(
			MessageContents::Header(format!("Updating profile {id}")),
			MessageLevel::Important,
		);

		let print_options = PrintOptions::new(true, 0);
		let mut manager = UpdateManager::new(print_options, force, false);
		manager
			.fulfill_version_manifest(&profile.version, paths, ctx.output)
			.await
			.context("Failed to get version information")?;
		let mc_version = manager.version_info.get().version.clone();

		let paper_properties = get_paper_properties(profile, &mc_version)
			.await
			.context("Failed to get Paper build number and filename")?;

		check_profile_version_change(profile, &mc_version, paper_properties.clone(), &mut ctx)
			.await
			.context("Failed to check for a profile version update")?;

		check_profile_paper_update(profile, paper_properties, &mut ctx)
			.await
			.context("Failed to check for Paper updates")?;

		ctx.lock
			.finish(paths)
			.await
			.context("Failed to finish using lockfile")?;

		if !update_packages {
			return Ok(());
		}

		if !profile.instances.is_empty() {
			let version_list = profile
				.create_instances(
					ctx.instances,
					paths,
					manager,
					ctx.lock,
					&config.users,
					ctx.output,
				)
				.await
				.context("Failed to create profile instances")?;

			if !profile.packages.is_empty() {
				ctx.output.display(
					MessageContents::Header("Updating packages".to_string()),
					MessageLevel::Important,
				);
			}

			// Make sure all packages in the profile are in the registry first
			for pkg in &profile.packages {
				ctx.packages.ensure_package(&pkg.req, paths).await?;
			}

			let constants = EvalConstants {
				version: mc_version.to_string(),
				modifications: profile.modifications.clone(),
				version_list: version_list.clone(),
				language: config.prefs.language,
			};

			let packages = update_profile_packages(profile, &constants, &mut ctx, force).await?;

			ctx.output.display(
				MessageContents::Success("All packages installed".to_string()),
				MessageLevel::Important,
			);

			all_packages.extend(packages);
		}

		ctx.lock
			.finish(paths)
			.await
			.context("Failed to finish using lockfile")?;
	}

	let all_packages = Vec::from_iter(all_packages);
	print_package_support_messages(&all_packages, &mut ctx)
		.await
		.context("Failed to print support messages")?;

	Ok(())
}

/// Update a profile when the Minecraft version has changed
async fn check_profile_version_change<'a, O: MCVMOutput>(
	profile: &Profile,
	mc_version: &str,
	paper_properties: Option<(u16, String)>,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<()> {
	if ctx.lock.update_profile_version(&profile.id, mc_version) {
		ctx.output.start_process();
		ctx.output.display(
			MessageContents::StartProcess("Updating profile version".to_string()),
			MessageLevel::Important,
		);

		for instance_id in profile.instances.iter() {
			let instance = ctx.instances.get(instance_id).ok_or(anyhow!(
				"Instance '{instance_id}' does not exist in the registry"
			))?;
			instance
				.teardown(ctx.paths, paper_properties.clone())
				.context("Failed to remove old files when updating Minecraft version")?;
		}

		ctx.output.display(
			MessageContents::Success("Profile version changed".to_string()),
			MessageLevel::Important,
		);
		ctx.output.end_process();
	}
	Ok(())
}

/// Get the updated Paper file name and build number for a profile that uses it
async fn get_paper_properties(
	profile: &Profile,
	mc_version: &str,
) -> anyhow::Result<Option<(u16, String)>> {
	let out = if let ServerType::Paper = profile.modifications.server_type {
		let (build_num, ..) = paper::get_newest_build(mc_version)
			.await
			.context("Failed to get the newest Paper build number")?;
		let paper_file_name = paper::get_jar_file_name(mc_version, build_num)
			.await
			.context("Failed to get the name of the Paper Jar file")?;
		Some((build_num, paper_file_name))
	} else {
		None
	};

	Ok(out)
}

/// Remove the old Paper files for a profile if they have updated
async fn check_profile_paper_update<'a, O: MCVMOutput>(
	profile: &Profile,
	paper_properties: Option<(u16, String)>,
	ctx: &mut ProfileUpdateContext<'a, O>,
) -> anyhow::Result<()> {
	if let Some((build_num, file_name)) = paper_properties {
		if ctx.lock.update_profile_paper_build(&profile.id, build_num) {
			for inst in profile.instances.iter() {
				if let Some(inst) = ctx.instances.get(inst) {
					inst.remove_paper(ctx.paths, file_name.clone())
						.context("Failed to remove Paper")?;
				}
			}
		}
	}

	Ok(())
}
