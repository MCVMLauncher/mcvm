pub mod conditions;
/// Evaluating declarative packages
pub mod declarative;
/// Evaluating script packages
pub mod script;

use anyhow::bail;
use anyhow::Context;
use async_trait::async_trait;
use color_print::cprintln;
use mcvm_parse::properties::PackageProperties;
use mcvm_parse::routine::INSTALL_ROUTINE;
use mcvm_pkg::resolve::ResolutionResult;
use mcvm_pkg::ConfiguredPackage;
use mcvm_pkg::PackageContentType;
use mcvm_pkg::PkgRequest;
use mcvm_pkg::RequiredPackage;
use mcvm_pkg::{
	PackageEvalRelationsResult as EvalRelationsResultTrait,
	PackageEvaluator as PackageEvaluatorTrait,
};
use mcvm_shared::addon::{is_addon_version_valid, is_filename_valid, Addon, AddonKind};
use mcvm_shared::lang::Language;
use mcvm_shared::util::is_valid_identifier;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use self::declarative::eval_declarative_package;
use self::script::eval_script_package;

use super::calculate_features;
use super::reg::PkgRegistry;
use super::Package;
use super::PkgProfileConfig;
use crate::data::addon::{self, AddonLocation, AddonRequest};
use crate::data::config::profile::GameModifications;
use crate::io::files::paths::Paths;
use crate::util::hash::{
	get_hash_str_as_hex, HASH_SHA256_RESULT_LENGTH, HASH_SHA512_RESULT_LENGTH,
};
use mcvm_shared::instance::Side;
use mcvm_shared::pkg::{PackageAddonOptionalHashes, PackageStability, PkgIdentifier};

use std::collections::HashMap;
use std::path::PathBuf;

/// Max notice instructions per package
static MAX_NOTICE_INSTRUCTIONS: usize = 10;
/// Max characters per notice instruction
static MAX_NOTICE_CHARACTERS: usize = 128;

/// What instructions the evaluator will evaluate (depends on what routine we are running)
#[derive(Debug, Clone)]
pub enum EvalLevel {
	/// When we are installing the addons of a package
	Install,
	/// When we are resolving package relationships
	Resolve,
}

/// Permissions level for an evaluation
#[derive(Deserialize, Serialize, Debug, Copy, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum EvalPermissions {
	Restricted,
	#[default]
	Standard,
	Elevated,
}

/// Context / purpose for when we are evaluating
pub enum Routine {
	Install,
	/// Install routine, except for resolution
	InstallResolve,
}

impl Routine {
	pub fn get_routine_name(&self) -> String {
		String::from(match self {
			Self::Install => INSTALL_ROUTINE,
			Self::InstallResolve => INSTALL_ROUTINE,
		})
	}

	pub fn get_level(&self) -> EvalLevel {
		match self {
			Self::Install => EvalLevel::Install,
			Self::InstallResolve => EvalLevel::Resolve,
		}
	}
}

/// Constants for the evaluation that will be the same across every package
#[derive(Debug, Clone)]
pub struct EvalConstants {
	pub version: String,
	pub modifications: GameModifications,
	pub version_list: Vec<String>,
	pub language: Language,
}

/// Constants for the evaluation that may be different for each package
#[derive(Debug, Clone)]
pub struct EvalParameters {
	pub side: Side,
	pub features: Vec<String>,
	pub perms: EvalPermissions,
	pub stability: PackageStability,
}

/// Combination of both EvalConstants and EvalParameters
#[derive(Debug, Clone)]
pub struct EvalInput<'a> {
	pub constants: &'a EvalConstants,
	pub params: EvalParameters,
}

/// Persistent state for evaluation
#[derive(Debug, Clone)]
pub struct EvalData<'a> {
	pub input: EvalInput<'a>,
	pub id: PkgIdentifier,
	pub level: EvalLevel,
	pub vars: HashMap<String, String>,
	pub addon_reqs: Vec<AddonRequest>,
	pub deps: Vec<Vec<RequiredPackage>>,
	pub conflicts: Vec<String>,
	pub recommendations: Vec<String>,
	pub bundled: Vec<String>,
	pub compats: Vec<(String, String)>,
	pub extensions: Vec<String>,
	pub notices: Vec<String>,
	pub commands: Vec<Vec<String>>,
}

impl<'a> EvalData<'a> {
	pub fn new(input: EvalInput<'a>, id: PkgIdentifier, routine: &Routine) -> Self {
		Self {
			input,
			id,
			level: routine.get_level(),
			vars: HashMap::new(),
			addon_reqs: Vec::new(),
			deps: Vec::new(),
			conflicts: Vec::new(),
			recommendations: Vec::new(),
			bundled: Vec::new(),
			compats: Vec::new(),
			extensions: Vec::new(),
			notices: Vec::new(),
			commands: Vec::new(),
		}
	}
}

impl Package {
	/// Evaluate a routine on a package
	pub async fn eval<'a>(
		&mut self,
		paths: &Paths,
		routine: Routine,
		input: EvalInput<'a>,
		client: &Client,
	) -> anyhow::Result<EvalData<'a>> {
		self.parse(paths, client).await?;

		// Check properties
		let properties = self.get_properties(paths, client).await?;
		if eval_check_properties(&input, properties)? {
			return Ok(EvalData::new(input, self.id.clone(), &routine));
		}

		match self.content_type {
			PackageContentType::Script => {
				let parsed = self.data.get_mut().contents.get_mut().get_script_contents();
				let eval = eval_script_package(self.id.clone(), parsed, routine, input)?;
				Ok(eval)
			}
			PackageContentType::Declarative => {
				let contents = self.data.get().contents.get().get_declarative_contents();
				let eval = eval_declarative_package(self.id.clone(), contents, input, routine)?;
				Ok(eval)
			}
		}
	}
}

/// Check properties when evaluating. Returns true if the package should finish evaluating with no error
pub fn eval_check_properties(
	input: &EvalInput,
	properties: &PackageProperties,
) -> anyhow::Result<bool> {
	if let Some(supported_modloaders) = &properties.supported_modloaders {
		if !supported_modloaders.iter().any(|x| {
			x.matches(
				&input
					.constants
					.modifications
					.get_modloader(input.params.side),
			)
		}) {
			bail!("Package does not support this modloader");
		}
	}
	if let Some(supported_plugin_loaders) = &properties.supported_plugin_loaders {
		if !supported_plugin_loaders
			.iter()
			.any(|x| x.matches(&input.constants.modifications.server_type))
		{
			bail!("Package does not support this plugin loader");
		}
	}

	if let Some(supported_sides) = &properties.supported_sides {
		if !supported_sides.contains(&input.params.side) {
			return Ok(true);
		}
	}

	Ok(false)
}

/// Utility for evaluation that validates addon arguments and creates a request
pub fn create_valid_addon_request(
	id: String,
	url: Option<String>,
	path: Option<String>,
	kind: AddonKind,
	file_name: Option<String>,
	version: Option<String>,
	pkg_id: PkgIdentifier,
	hashes: PackageAddonOptionalHashes,
	eval_input: &EvalInput,
) -> anyhow::Result<AddonRequest> {
	if !is_valid_identifier(&id) {
		bail!("Invalid addon identifier '{id}'");
	}

	// Empty strings will break the filename so we convert them to none
	let version = version.filter(|x| !x.is_empty());
	if let Some(version) = &version {
		if !is_addon_version_valid(version) {
			bail!("Invalid addon version identifier '{version}' for addon '{id}'");
		}
	}

	let file_name =
		file_name.unwrap_or(addon::get_addon_instance_filename(&pkg_id.name, &id, &kind));

	if !is_filename_valid(kind, &file_name) {
		bail!("Invalid addon filename '{file_name}' in addon '{id}'");
	}

	// Check hashes
	if let Some(hash) = &hashes.sha256 {
		let hex = get_hash_str_as_hex(hash).context("Failed to parse hash string")?;
		if hex.len() > HASH_SHA256_RESULT_LENGTH {
			bail!("SHA-256 hash for addon '{id}' is longer than {HASH_SHA256_RESULT_LENGTH} characters");
		}
	}

	if let Some(hash) = &hashes.sha512 {
		let hex = get_hash_str_as_hex(hash).context("Failed to parse hash string")?;
		if hex.len() > HASH_SHA512_RESULT_LENGTH {
			bail!("SHA-512 hash for addon '{id}' is longer than {HASH_SHA512_RESULT_LENGTH} characters");
		}
	}

	let addon = Addon {
		kind,
		id: id.clone(),
		file_name,
		pkg_id,
		version,
		hashes,
	};

	if let Some(url) = url {
		let location = AddonLocation::Remote(url);
		Ok(AddonRequest::new(addon, location))
	} else if let Some(path) = path {
		match eval_input.params.perms {
			EvalPermissions::Elevated => {
				let path = String::from(shellexpand::tilde(&path));
				let path = PathBuf::from(path);
				let location = AddonLocation::Local(path);
				Ok(AddonRequest::new(addon, location))
			}
			_ => {
				bail!("Insufficient permissions to add a local addon '{id}'");
			}
		}
	} else {
		bail!("No location (url/path) was specified for addon '{id}'");
	}
}

/// Evaluator used as an input for dependency resolution
struct PackageEvaluator<'a> {
	reg: &'a mut PkgRegistry,
}

/// Common argument for the evaluator
struct EvaluatorCommonInput<'a> {
	paths: &'a Paths,
	client: &'a Client,
}

/// Newtype for PkgProfileConfig
#[derive(Clone)]
struct PackageConfig(PkgProfileConfig);

impl ConfiguredPackage for PackageConfig {
	type EvalInput<'a> = EvalInput<'a>;

	fn get_package(&self) -> &PkgRequest {
		&self.0.req
	}

	fn override_configured_package_input(
		&self,
		properties: &PackageProperties,
		input: &mut Self::EvalInput<'_>,
	) -> anyhow::Result<()> {
		let features =
			calculate_features(&self.0, properties).context("Failed to calculate features")?;

		input.params.features = features;
		input.params.perms = self.0.permissions;
		input.params.stability = self.0.stability;

		Ok(())
	}
}

struct EvalRelationsResult {
	pub deps: Vec<Vec<RequiredPackage>>,
	pub conflicts: Vec<String>,
	pub recommendations: Vec<String>,
	pub bundled: Vec<String>,
	pub compats: Vec<(String, String)>,
	pub extensions: Vec<String>,
}

impl EvalRelationsResultTrait for EvalRelationsResult {
	fn get_bundled(&self) -> Vec<String> {
		self.bundled.clone()
	}

	fn get_compats(&self) -> Vec<(String, String)> {
		self.compats.clone()
	}

	fn get_conflicts(&self) -> Vec<String> {
		self.conflicts.clone()
	}

	fn get_deps(&self) -> Vec<Vec<RequiredPackage>> {
		self.deps.clone()
	}
	fn get_extensions(&self) -> Vec<String> {
		self.extensions.clone()
	}

	fn get_recommendations(&self) -> Vec<String> {
		self.recommendations.clone()
	}
}

#[async_trait]
impl<'a> PackageEvaluatorTrait<'a> for PackageEvaluator<'a> {
	type CommonInput = EvaluatorCommonInput<'a>;
	type ConfiguredPackage = PackageConfig;
	type EvalInput<'b> = EvalInput<'b>;
	type EvalRelationsResult<'b> = EvalRelationsResult;

	async fn eval_package_relations(
		&mut self,
		pkg: &PkgRequest,
		input: &Self::EvalInput<'a>,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<Self::EvalRelationsResult<'a>> {
		let eval = self
			.reg
			.eval(
				pkg,
				common_input.paths,
				Routine::InstallResolve,
				input.clone(),
				common_input.client,
			)
			.await
			.context("Failed to evaluate dependencies for package")?;

		let result = EvalRelationsResult {
			deps: eval.deps,
			conflicts: eval.conflicts,
			recommendations: eval.recommendations,
			bundled: eval.bundled,
			compats: eval.compats,
			extensions: eval.extensions,
		};

		Ok(result)
	}

	async fn get_package_properties<'b>(
		&'b mut self,
		pkg: &PkgRequest,
		common_input: &Self::CommonInput,
	) -> anyhow::Result<&'b PackageProperties> {
		let properties = self
			.reg
			.get_properties(pkg, common_input.paths, common_input.client)
			.await?;
		Ok(properties)
	}
}

/// Resolve package dependencies
pub async fn resolve(
	packages: &[PkgProfileConfig],
	constants: &EvalConstants,
	default_params: EvalParameters,
	paths: &Paths,
	reg: &mut PkgRegistry,
) -> anyhow::Result<ResolutionResult> {
	let evaluator = PackageEvaluator { reg };

	let input = EvalInput {
		constants,
		params: default_params,
	};

	let client = Client::new();
	let common_input = EvaluatorCommonInput {
		client: &client,
		paths,
	};

	let packages = packages
		.iter()
		.map(|x| PackageConfig(x.clone()))
		.collect::<Vec<_>>();

	let result = mcvm_pkg::resolve::resolve(&packages, evaluator, input, &common_input).await?;

	for package in &result.unfulfilled_recommendations {
		let source = package.source.get_source();
		if let Some(source) = source {
			cprintln!(
				"<y>Warning: The package '{}' recommends the use of the package '{}', which is not installed.",
				source.debug_sources(String::new()),
				package
			);
		} else {
			cprintln!(
				"<y>Warning: A package recommends the use of the package '{}', which is not installed.",
				package
			);
		}
	}

	Ok(result)
}
