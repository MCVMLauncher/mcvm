use super::lib::{CmdData, CmdError};
use crate::io::lock::Lockfile;
use crate::io::lock::LockfileAsset;
use crate::net::game_files::get_version_manifest;
use crate::net::game_files::make_version_list;
use crate::package::eval::eval::Routine;
use crate::package::eval::eval::EvalConstants;
use crate::data::instance::InstKind;
use crate::util::print::HYPHEN_POINT;
use crate::util::print::ReplPrinter;

use color_print::cformat;
use color_print::{cprintln, cprint};

static INFO_HELP: &str = "View helpful information about a profile";
static LIST_HELP: &str = "List all profiles and their instances";
static UPDATE_HELP: &str = "Update the packages and instances of a profile";
static REINSTALL_HELP: &str = "Force reinstall a profile and all its files";

pub fn help() {
	cprintln!("<i>profile:</i> Manage mcvm profiles");
	cprintln!("<s>Usage:</s> mcvm profile <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>info:</i,c> {}", HYPHEN_POINT, INFO_HELP);
	cprintln!("{}<i,c>list:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>update:</i,c> {}", HYPHEN_POINT, UPDATE_HELP);
	cprintln!("{}<i,c>reinstall:</i,c> {}", HYPHEN_POINT, REINSTALL_HELP);
}

fn info(data: &mut CmdData, id: &String) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get(id) {
				cprintln!("<s><g>Profile <b>{}", id);
				cprintln!("   <s>Version:</s> <g>{}", profile.version);
				cprintln!("   <s>Modloader:</s> <g>{}", profile.modloader);
				cprintln!("   <s>Plugin Loader:</s> <g>{}", profile.pluginloader);
				cprintln!("   <s>Instances:");
				for inst_id in profile.instances.iter() {
					if let Some(instance) = config.instances.get(inst_id) {
						cprint!("   {}", HYPHEN_POINT);
						match instance.kind {
							InstKind::Client => cprint!("<y!>Client {}", inst_id),
							InstKind::Server => cprint!("<c!>Server {}", inst_id)
						}
						cprintln!();
					}
				}
				cprintln!("   <s>Packages:");
				for pkg in profile.packages.iter() {
					cprint!("   {}", HYPHEN_POINT);
					cprint!("<b!>{}:<g!>{}", pkg.req.name, config.packages.get_version(&pkg.req, paths)?);
					cprintln!();
				}
			} else {
				return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
			}
		}
	}
	Ok(())
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Profiles:");
		for (id, profile) in config.profiles.iter() {
			cprintln!("<s><g>   {}", id);
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					match instance.kind {
						InstKind::Client => cprintln!("   {}<y!>{}", HYPHEN_POINT, inst_id),
						InstKind::Server => cprintln!("   {}<c!>{}", HYPHEN_POINT, inst_id)
					}
				}
			}
		}
	}
	Ok(())
}

async fn profile_update(data: &mut CmdData, id: &String, force: bool) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get_mut(id) {
				cprintln!("<s>Obtaining version index...");
				let (version_manifest, ..) = get_version_manifest(paths)?;
				profile.create_instances(&mut config.instances, &version_manifest, paths, true, force).await?;
				
				cprintln!("<s>Updating packages");
				let mut printer = ReplPrinter::new(true);
				let mut lock = Lockfile::open(paths)?;
				let mut assets = Vec::new();
				for pkg in profile.packages.iter() {
					let version = config.packages.get_version(&pkg.req, paths)?;
					for instance_id in profile.instances.iter() {
						if let Some(instance) = config.instances.get(instance_id) {
							printer.print(&cformat!("\t(<b!>{}</b!>) Evaluating...", pkg.req));
							let constants = EvalConstants {
								version: profile.version.clone(),
								modloader: profile.modloader.clone(),
								pluginloader: profile.pluginloader.clone(),
								side: instance.kind.clone(),
								features: pkg.features.clone(),
								versions: make_version_list(&version_manifest)?
							};
							let eval = config.packages.eval(&pkg.req, paths, Routine::Install, constants).await?;
							printer.print(&cformat!("\t(<b!>{}</b!>) Downloading files...", pkg.req));
							for asset in eval.downloads.iter() {
								asset.download(paths).await?;
								instance.create_asset(&asset.asset, paths)?;
								assets.push(
									LockfileAsset::from_asset(&asset.asset, paths)
								);
							}
							let assets_to_remove = lock.update_package(&pkg.req.name, instance_id, &version, &assets)?;
							for asset in eval.downloads.iter() {
								if assets_to_remove.contains(&asset.asset.name) {
									instance.remove_asset(&asset.asset, paths)?;
								}
							}

							printer.newline();
						}
					}
				}
				
				for instance_id in profile.instances.iter() {
					if let Some(instance) = config.instances.get(instance_id) {
						let assets_to_remove = lock.remove_unused_packages(
							instance_id,
							&profile.packages.iter().map(|x| x.req.name.clone())
								.collect::<Vec<String>>()
						)?;

						for asset in assets_to_remove {
							instance.remove_asset(&asset, paths)?;
						}
					}
				}

				lock.finish(paths)?;

				printer.print(&cformat!("\t<g>Finished installing packages."));
				printer.finish();
			} else {
				return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
			}
		}
	}
	Ok(())
}

pub async fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"list" => list(data)?,
		"info" => match argc {
			1 => cprintln!("{}", INFO_HELP),
			_ => info(data, &argv[1])?
		}
		"update" => match argc {
			1 => cprintln!("{}", UPDATE_HELP),
			_ => profile_update(data, &argv[1], false).await?
		}
		"reinstall" => match argc {
			1 => cprintln!("{}", REINSTALL_HELP),
			_ => profile_update(data, &argv[1], true).await?
		}
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
