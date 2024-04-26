mod backup;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use backup::{get_backup_directory, Config, Index, DEFAULT_GROUP};
use clap::Parser;
use color_print::cprintln;
use mcvm_plugin::api::{CustomPlugin, HookContext};
use mcvm_plugin::hooks;
use mcvm_shared::id::InstanceRef;

use crate::backup::BackupSource;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("backup")?;
	plugin.subcommand(|ctx, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "backup" && subcommand != "back" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);
		let result = match cli.command {
			Subcommand::List {
				raw,
				instance,
				group,
			} => list(&ctx, raw, &instance, group.as_deref()),
			Subcommand::Create { instance, group } => create(&ctx, &instance, group.as_deref()),
			Subcommand::Remove {
				instance,
				group,
				backup,
			} => remove(&ctx, &instance, group.as_deref(), &backup),
			Subcommand::Restore {
				instance,
				group,
				backup,
			} => restore(&ctx, &instance, group.as_deref(), &backup),
			Subcommand::Info {
				instance,
				group,
				backup,
			} => info(&ctx, &instance, group.as_deref(), &backup),
		};
		result?;

		Ok(())
	})?;

	Ok(())
}

#[derive(clap::Parser)]
struct Cli {
	#[command(subcommand)]
	command: Subcommand,
}

#[derive(clap::Subcommand)]
#[command(name = "mcvm backup")]
enum Subcommand {
	#[command(about = "List backups for an instance")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// The instance to list backups for
		instance: String,
		/// The group to list backups for
		group: Option<String>,
	},
	#[command(about = "Create a backup")]
	Create {
		/// The instance to create a backup for
		instance: String,
		/// The group to create the backup for
		group: Option<String>,
	},
	#[command(about = "Remove an existing backup")]
	Remove {
		/// The instance the backup is in
		instance: String,
		/// The group the backup is in
		group: Option<String>,
		/// The backup to remove
		backup: String,
	},
	#[command(about = "Restore an existing backup")]
	Restore {
		/// The instance the backup is in
		instance: String,
		/// The group the backup is in
		group: Option<String>,
		/// The backup to restore
		backup: String,
	},
	#[command(about = "Print information about a specific backup")]
	Info {
		/// The instance the backup is in
		instance: String,
		/// The group the backup is in
		group: Option<String>,
		/// The backup to get info about
		backup: String,
	},
}

fn list(
	ctx: &HookContext<'_, hooks::Subcommand>,
	raw: bool,
	instance: &str,
	group: Option<&str>,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, &inst_ref)?;
	let group = index
		.contents
		.groups
		.get(group)
		.context("Group does not exist")?;

	for backup in &group.backups {
		if raw {
			println!("{}", backup.id);
		} else {
			cprintln!("<k!> - </>{}", backup.id);
		}
	}

	index.finish()?;
	Ok(())
}

fn create(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let mut index = get_index(ctx, &inst_ref)?;

	let inst_dir = ctx
		.get_data_dir()?
		.join("instances")
		.join(inst_ref.profile.to_string())
		.join(&inst_ref.instance.to_string());

	index.create_backup(BackupSource::User, Some(group), &inst_dir)?;

	index.finish()?;

	cprintln!("<g>Backup created.");

	Ok(())
}

fn remove(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	backup: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let mut index = get_index(ctx, &inst_ref)?;

	index.remove_backup(group, backup)?;
	index.finish()?;

	cprintln!("<g>Backup removed.");

	Ok(())
}

fn restore(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	backup: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, &inst_ref)?;

	let inst_dir = ctx
		.get_data_dir()?
		.join("instances")
		.join(inst_ref.profile.to_string())
		.join(&inst_ref.instance.to_string());

	index.restore_backup(group, backup, &inst_dir)?;
	index.finish()?;

	cprintln!("<g>Backup restored.");

	Ok(())
}

fn info(
	ctx: &HookContext<'_, hooks::Subcommand>,
	instance: &str,
	group: Option<&str>,
	backup_id: &str,
) -> anyhow::Result<()> {
	let inst_ref =
		InstanceRef::parse(instance.into()).context("Failed to parse instance reference")?;
	let group = group.unwrap_or(DEFAULT_GROUP);

	let index = get_index(ctx, &inst_ref)?;

	let backup = index.get_backup(group, backup_id)?;

	cprintln!(
		"<s>Backup <b>{}</b> in instance <g>{}</g>:",
		backup_id,
		inst_ref
	);
	cprintln!("<k!> - </>Date created: <c>{}", backup.date);

	Ok(())
}

fn get_index(
	ctx: &HookContext<'_, hooks::Subcommand>,
	inst_ref: &InstanceRef,
) -> anyhow::Result<Index> {
	let dir = get_backup_directory(&get_backups_dir(ctx)?, inst_ref);
	Index::open(&dir, inst_ref.clone(), &get_backup_config(inst_ref, ctx)?)
}

fn get_backups_dir(ctx: &HookContext<'_, hooks::Subcommand>) -> anyhow::Result<PathBuf> {
	let dir = ctx.get_data_dir()?.join("backups");
	std::fs::create_dir_all(&dir)?;
	Ok(dir)
}

fn get_backup_config(
	instance: &InstanceRef,
	ctx: &HookContext<'_, hooks::Subcommand>,
) -> anyhow::Result<Config> {
	let config = ctx.get_custom_config().unwrap_or("{}");
	let mut config: HashMap<String, Config> =
		serde_json::from_str(config).context("Failed to deserialize custom config")?;
	let config = config.remove(&instance.to_string()).unwrap_or_default();
	Ok(config)
}
