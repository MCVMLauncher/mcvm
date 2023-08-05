use crate::{instruction::InstrKind, parse::Parsed, routine::METADATA_ROUTINE};
use anyhow::bail;

/// Package metadata derived from running the 'meta' routine
#[derive(Default, Debug)]
pub struct PackageMetadata {
	pub name: Option<String>,
	pub description: Option<String>,
	pub long_description: Option<String>,
	pub version: Option<String>,
	pub authors: Option<Vec<String>>,
	pub package_maintainers: Option<Vec<String>>,
	pub website: Option<String>,
	pub support_link: Option<String>,
	pub documentation: Option<String>,
	pub source: Option<String>,
	pub issues: Option<String>,
	pub community: Option<String>,
	pub icon: Option<String>,
	pub banner: Option<String>,
	pub license: Option<String>,
}

/// Collect the metadata from a package
pub fn eval_metadata(parsed: &Parsed) -> anyhow::Result<PackageMetadata> {
	if let Some(routine_id) = parsed.routines.get(METADATA_ROUTINE) {
		if let Some(block) = parsed.blocks.get(routine_id) {
			let mut out = PackageMetadata::default();

			for instr in &block.contents {
				match &instr.kind {
					InstrKind::Name(val) => out.name = Some(val.get_clone()),
					InstrKind::Description(val) => out.description = Some(val.get_clone()),
					InstrKind::LongDescription(val) => out.long_description = Some(val.get_clone()),
					InstrKind::Version(val) => out.version = Some(val.get_clone()),
					InstrKind::Authors(val) => out.authors = Some(val.clone()),
					InstrKind::PackageMaintainers(val) => {
						out.package_maintainers = Some(val.clone())
					}
					InstrKind::Website(val) => out.website = Some(val.get_clone()),
					InstrKind::SupportLink(val) => out.support_link = Some(val.get_clone()),
					InstrKind::Documentation(val) => out.documentation = Some(val.get_clone()),
					InstrKind::Source(val) => out.source = Some(val.get_clone()),
					InstrKind::Issues(val) => out.issues = Some(val.get_clone()),
					InstrKind::Community(val) => out.community = Some(val.get_clone()),
					InstrKind::Icon(val) => out.icon = Some(val.get_clone()),
					InstrKind::Banner(val) => out.banner = Some(val.get_clone()),
					InstrKind::License(val) => out.license = Some(val.get_clone()),
					_ => bail!("Instruction is not allowed in this context"),
				}
			}

			Ok(out)
		} else {
			Ok(PackageMetadata::default())
		}
	} else {
		Ok(PackageMetadata::default())
	}
}
