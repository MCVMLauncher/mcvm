use anyhow::{anyhow, bail, Context};
use clap::Subcommand;
use color_print::cprintln;
use itertools::Itertools;
use mcvm::data::user::AuthState;

use mcvm::io::lock::Lockfile;
use mcvm::{data::instance::InstKind, util::print::HYPHEN_POINT};
use mcvm_shared::instance::Side;

use super::CmdData;
use crate::cli::get_ms_client_id;

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances in all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// Filter by instance side
		#[arg(short, long)]
		side: Option<Side>,
		/// Filter by profile
		#[arg(short, long)]
		profile: Option<String>,
	},
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// Whether to print the command that was generated when launching
		#[arg(short, long)]
		debug: bool,
		/// An optional user to choose when launching
		#[arg(short, long)]
		user: Option<String>,
		/// The instance to launch
		instance: String,
	},
}

async fn list(
	data: &mut CmdData,
	raw: bool,
	side: Option<Side>,
	profile: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let profile = if let Some(profile) = profile {
		Some(
			config
				.profiles
				.get(&profile)
				.ok_or(anyhow!("Profile '{profile}' does not exist"))?,
		)
	} else {
		None
	};

	for (id, instance) in config.instances.iter().sorted_by_key(|x| x.0) {
		if let Some(side) = side {
			if instance.kind.to_side() != side {
				continue;
			}
		}

		if let Some(profile) = profile {
			if !profile.instances.contains(id) {
				continue;
			}
		}

		if raw {
			println!("{id}");
		} else {
			match instance.kind {
				InstKind::Client { .. } => cprintln!("{}<y!>{}", HYPHEN_POINT, id),
				InstKind::Server { .. } => cprintln!("{}<c!>{}", HYPHEN_POINT, id),
			}
		}
	}

	Ok(())
}

pub async fn launch(
	instance: &str,
	debug: bool,
	user: Option<String>,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config(true).await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	let instance = config
		.instances
		.get_mut(instance)
		.ok_or(anyhow!("Unknown instance '{instance}'"))?;
	let (.., profile) = config
		.profiles
		.iter()
		.find(|(.., profile)| profile.instances.contains(&instance.id))
		.expect("Instance does not belong to any profiles");

	if let Some(user) = user {
		if !config.users.users.contains_key(&user) {
			bail!("User '{user}' does not exist");
		}
		config.users.state = AuthState::UserChosen(user);
	}

	if let InstKind::Client { .. } = &instance.kind {
		config
			.users
			.ensure_authenticated(get_ms_client_id())
			.await
			.context("Failed to authenticate user")?;
	}

	let mut lock = Lockfile::open(paths)?;

	instance
		.launch(
			paths,
			&mut lock,
			&config.users,
			debug,
			&profile.version,
		)
		.await
		.context("Instance failed to launch")?;

	Ok(())
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw, side, profile } => list(data, raw, side, profile).await,
		InstanceSubcommand::Launch {
			debug,
			user,
			instance,
		} => launch(&instance, debug, user, data).await,
	}
}
