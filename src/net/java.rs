use crate::net::download;
use crate::util::json::{self, JsonType};
use crate::util::{ARCH_STRING, OS_STRING, PREFERRED_ARCHIVE};

use anyhow::{anyhow, Context};

pub mod adoptium {
	use super::*;

	/// Gets the URL to the JSON file for a major Java version
	fn json_url(major_version: &str) -> String {
		format!(
			"https://api.adoptium.net/v3/assets/latest/{major_version}/hotspot?image_type=jre&vendor=eclipse&architecture={ARCH_STRING}&os={OS_STRING}"
		)
	}

	/// Gets the newest Adoptium binaries download for a major Java version
	pub async fn get_latest(major_version: &str) -> anyhow::Result<json::JsonObject> {
		let url = json_url(major_version);
		let manifest = download::json::<serde_json::Value>(&url)
			.await
			.context("Failed to download manifest of Adoptium versions")?;
		let manifest = json::ensure_type(manifest.as_array(), JsonType::Arr)
			.context("Expected manifest to be an array of versions")?;
		let version = json::ensure_type(
			manifest
				.get(0)
				.ok_or(anyhow!("A valid installation was not found"))?
				.as_object(),
			JsonType::Obj,
		)?;

		Ok(version.to_owned())
	}
}

pub mod zulu {
	use super::*;
	
	use crate::util::preferred_archive_extension;
	use serde::Deserialize;

	/// Gets the URL to the JSON file for a major Java version
	fn json_url(major_version: &str) -> String {
		format!(
			"https://api.azul.com/metadata/v1/zulu/packages/?java_version={major_version}&os={OS_STRING}&arch={ARCH_STRING}&archive_type={PREFERRED_ARCHIVE}&java_package_type=jre&latest=true&java_package_features=headfull&release_status=ga&availability_types=CA&certifications=tck&page=1&page_size=100"
		)
	}

	/// Format of the metadata JSON with download info for Zulu
	#[derive(Deserialize, Clone)]
	pub struct PackageFormat {
		pub name: String,
		pub download_url: String,
	}

	/// Gets the newest Zulu package for a major Java version
	pub async fn get_latest(major_version: &str) -> anyhow::Result<PackageFormat> {
		let url = json_url(major_version);
		let manifest = download::json::<Vec<PackageFormat>>(&url)
			.await
			.context("Failed to download manifest of Zulu versions")?;
		let package = manifest
			.get(0)
			.ok_or(anyhow!("A valid installation was not found"))?;

		Ok(package.to_owned())
	}

	/// Gets the name of the extracted directory by removing the archive file extension
	pub fn extract_dir_name(name: &str) -> String {
		name.replacen(&preferred_archive_extension(), "", 1)
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_extract_dir_name() {
			let name = format!("hello.{PREFERRED_ARCHIVE}");
			assert_eq!(extract_dir_name(&name), "hello");
		}
	}
}
