use crate::data::instance::InstKind;
use crate::data::instance::create::CreateError;
use crate::data::profile::update::UpdateManager;
use crate::io::files::{self, paths::Paths};
use crate::io::java::classpath::Classpath;
use crate::util::json::{self, JsonObject, JsonType};
use crate::util::{mojang, cap_first_letter};
use crate::util::print::ReplPrinter;
use crate::util::versions::VersionNotFoundError;

use color_print::{cformat, cprintln};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use zip::ZipArchive;

use std::collections::HashSet;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::download::{FD_SENSIBLE_LIMIT, download_text, download_bytes};

#[derive(Debug, thiserror::Error)]
pub enum VersionManifestError {
	#[error("Failed to download:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
}

// So we can do this without a retry
async fn get_version_manifest_contents(paths: &Paths) -> Result<String, VersionManifestError> {
	let mut path = paths.internal.join("versions");
	files::create_dir(&path)?;
	path.push("manifest.json");

	let text = download_text("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json").await?;
	fs::write(&path, &text)?;

	Ok(text)
}

pub async fn get_version_manifest(
	paths: &Paths,
) -> Result<Box<json::JsonObject>, VersionManifestError> {
	let mut manifest_contents = get_version_manifest_contents(paths).await?;
	let manifest = match json::parse_object(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(..) => {
			cprintln!("<r>Failed to parse version manifest. Redownloading...");
			manifest_contents = get_version_manifest_contents(paths).await?;
			json::parse_object(&manifest_contents)?
		}
	};
	Ok(manifest)
}

// Makes an ordered list of versions from the manifest to use for matching
pub fn make_version_list(
	version_manifest: &json::JsonObject,
) -> Result<Vec<String>, VersionManifestError> {
	let versions = json::access_array(version_manifest, "versions")?;
	let mut out = Vec::new();
	for entry in versions {
		let entry_obj = json::ensure_type(entry.as_object(), JsonType::Obj)?;
		out.push(json::access_str(entry_obj, "id")?.to_owned());
	}
	out.reverse();
	Ok(out)
}

#[derive(Debug, thiserror::Error)]
pub enum VersionJsonError {
	#[error("Version {} does not exist", .0)]
	VersionNotFound(#[from] VersionNotFoundError),
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("{}", .0)]
	VersionManifest(#[from] VersionManifestError),
	#[error("Error when downloading file:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
}

pub async fn get_version_json(
	version: &str,
	version_manifest: &json::JsonObject,
	paths: &Paths,
) -> Result<Box<json::JsonObject>, VersionJsonError> {
	let version_string = version.to_owned();

	// Find the version out of all of them
	let versions = json::access_array(version_manifest, "versions")?;
	let mut version_url: Option<&str> = None;
	for entry in versions.iter() {
		let obj = json::ensure_type(entry.as_object(), JsonType::Obj)?;
		if json::access_str(obj, "id")? == version_string {
			version_url = Some(json::access_str(obj, "url")?);
		}
	}
	if version_url.is_none() {
		return Err(VersionJsonError::from(VersionNotFoundError::new(version)));
	}

	let version_json_name: String = version_string.clone() + ".json";
	let version_folder = paths.internal.join("versions").join(version_string);
	files::create_dir(&version_folder)?;
	let text = download_text(version_url.expect("Version does not exist")).await?;
	tokio::fs::write(version_folder.join(version_json_name), &text).await?;

	let version_doc = json::parse_object(&text)?;

	Ok(version_doc)
}

#[derive(Debug, thiserror::Error)]
pub enum LibrariesError {
	#[error("Failed to evaluate json file:\n{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Error when downloading library:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to access zip file:\n{}", .0)]
	Zip(#[from] zip::result::ZipError),
}

// Checks the rules of a library to see if it should be installed
fn is_library_allowed(lib: &JsonObject) -> Result<bool, LibrariesError> {
	if let Some(rules_val) = lib.get("rules") {
		let rules = json::ensure_type(rules_val.as_array(), JsonType::Arr)?;
		for rule_val in rules.iter() {
			let rule = json::ensure_type(rule_val.as_object(), JsonType::Obj)?;
			let action = json::access_str(rule, "action")?;
			if let Some(os_val) = rule.get("os") {
				let os = json::ensure_type(os_val.as_object(), JsonType::Obj)?;
				let os_name = json::access_str(os, "name")?;
				if mojang::is_allowed(action) != (os_name == mojang::OS_STRING) {
					return Ok(false);
				}
			}
		}
	}
	Ok(true)
}

// Finishes up and downloads a library
async fn download_library(
	client: &Client,
	lib_download: &json::JsonObject,
	path: &Path,
) -> Result<(), LibrariesError> {
	files::create_leading_dirs(path)?;
	let url = json::access_str(lib_download, "url")?;
	tokio::fs::write(path, client.get(url).send().await?.error_for_status()?.bytes().await?).await?;

	Ok(())
}

fn extract_native_library(path: &Path, natives_dir: &Path) -> Result<(), LibrariesError> {
	let file = File::open(path)?;
	let mut zip = ZipArchive::new(file)?;
	for i in 0..zip.len() {
		let mut file = zip.by_index(i)?;
		let rel_path = PathBuf::from(file.enclosed_name().expect("Invalid compressed file path"));
		if let Some(extension) = rel_path.extension() {
			match extension.to_str() {
				Some("so" | "dylib" | "dll") => {
					let mut out_file = File::create(natives_dir.join(rel_path))?;
					std::io::copy(&mut file, &mut out_file)?;
				}
				_ => continue,
			}
		}
	}

	Ok(())
}

/// Downloads base client libraries.
/// Returns a set of files to be added to the update manager.
pub async fn get_libraries(
	version_json: &json::JsonObject,
	paths: &Paths,
	version: &str,
	manager: &UpdateManager,
) -> Result<HashSet<PathBuf>, LibrariesError> {
	let mut files = HashSet::new();
	let libraries_path = paths.internal.join("libraries");
	files::create_dir(&libraries_path)?;
	let natives_path = paths
		.internal
		.join("versions")
		.join(version)
		.join("natives");
	files::create_dir(&natives_path)?;
	let natives_jars_path = paths.internal.join("natives");

	let mut native_paths = Vec::new();
	let client = Client::new();
	let mut printer = ReplPrinter::from_options(manager.print.clone());

	let libraries = json::access_array(version_json, "libraries")?;
	cprintln!(
		"Downloading <b>{}</> libraries...",
		libraries.len()
	);

	for lib_val in libraries.iter() {
		let lib = json::ensure_type(lib_val.as_object(), JsonType::Obj)?;
		if !is_library_allowed(lib)? {
			continue;
		}
		let name = json::access_str(lib, "name")?;
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives_val) = lib.get("natives") {
			let natives = json::ensure_type(natives_val.as_object(), JsonType::Obj)?;
			let key = json::access_str(natives, mojang::OS_STRING)?;
			let classifier =
				json::access_object(json::access_object(downloads, "classifiers")?, key)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);

			native_paths.push((path.clone(), name.to_owned()));
			if !manager.should_update_file(&path) {
				continue;
			}
			printer.print(&cformat!("Downloading library <b!>{}</>...", name));
			download_library(&client, classifier, &path).await?;
			files.insert(path);
			continue;
		}
		if let Some(artifact_val) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact_val.as_object(), JsonType::Obj)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			if !manager.should_update_file(&path) {
				continue;
			}
			printer.print(&cformat!("Downloading library <b>{}</>...", name));
			download_library(&client, artifact, &path).await?;
			files.insert(path);
			continue;
		}
	}

	for (path, name) in native_paths {
		printer.print(&cformat!("Extracting library <b!>{}...", name));
		extract_native_library(&path, &natives_path)?;
	}

	printer.print(&cformat!("<g>Libraries downloaded."));
	printer.finish();

	Ok(files)
}

/// Gets the classpath from Minecraft libraries
pub fn get_lib_classpath(
	version_json: &json::JsonObject,
	paths: &Paths
) -> Result<Classpath, LibrariesError> {
	let natives_jars_path = paths.internal.join("natives");
	let libraries_path = paths.internal.join("libraries");

	let mut classpath = Classpath::new();
	let libraries = json::access_array(version_json, "libraries")?;
	for lib_val in libraries.iter() {
		let lib = json::ensure_type(lib_val.as_object(), JsonType::Obj)?;
		if !is_library_allowed(lib)? {
			continue;
		}
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives_val) = lib.get("natives") {
			let natives = json::ensure_type(natives_val.as_object(), JsonType::Obj)?;
			let key = json::access_str(natives, mojang::OS_STRING)?;
			let classifier =
				json::access_object(json::access_object(downloads, "classifiers")?, key)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);
			classpath.add_path(&path);

			continue;
		}
		if let Some(artifact_val) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact_val.as_object(), JsonType::Obj)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			classpath.add_path(&path);
			continue;
		}
	}
	Ok(classpath)
}

#[derive(Debug, thiserror::Error)]
pub enum AssetsError {
	#[error("Failed to evaluate json file: {}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Error when downloading asset:\n{}", .0)]
	Download(#[from] reqwest::Error),
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to join tasks:\n{}", .0)]
	Join(#[from] tokio::task::JoinError),
}

async fn download_asset_index(url: &str, path: &Path) -> Result<Box<json::JsonObject>, AssetsError> {
	let text = download_text(url).await?;
	fs::write(path, &text)?;

	let doc = json::parse_object(&text)?;
	Ok(doc)
}

/// Download assets used by the client, such as game resources and icons.
pub async fn get_assets(
	version_json: &json::JsonObject,
	paths: &Paths,
	version: &str,
	manager: &UpdateManager,
) -> Result<HashSet<PathBuf>, AssetsError> {
	let mut out = HashSet::new();
	let version_string = version.to_owned();
	let indexes_dir = paths.assets.join("indexes");
	files::create_dir(&indexes_dir)?;

	let index_path = indexes_dir.join(version_string + ".json");
	let index_url = json::access_str(json::access_object(version_json, "assetIndex")?, "url")?;

	let objects_dir = paths.assets.join("objects");
	files::create_dir(&objects_dir)?;
	let virtual_dir = paths.assets.join("virtual");
	if !manager.force && virtual_dir.exists() && !virtual_dir.is_symlink() {
		files::dir_symlink(&virtual_dir, &objects_dir)?;
	}

	let index = match download_asset_index(index_url, &index_path).await {
		Ok(val) => val,
		Err(err) => {
			cprintln!(
				"<r>Failed to obtain asset index:\n{}\nRedownloading...",
				err
			);
			download_asset_index(index_url, &index_path).await?
		}
	};

	let assets = json::access_object(&index, "objects")?.clone();

	let client = Client::new();
	let mut join = JoinSet::new();
	let mut printer = ReplPrinter::from_options(manager.print.clone());
	let count = assets.len();
	if manager.print.verbose {
		cprintln!("Downloading <b>{}</> assets...", count);
	}
	let mut num_done = 0;
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	for (key, asset_val) in assets {
		let asset = json::ensure_type(asset_val.as_object(), JsonType::Obj)?;

		let hash = json::access_str(asset, "hash")?.to_owned();
		let hash_path = hash[..2].to_owned() + "/" + &hash;
		let url = "https://resources.download.minecraft.net/".to_owned() + &hash_path;

		let path = objects_dir.join(&hash_path);
		if !manager.should_update_file(&path) {
			continue;
		}
		
		out.insert(path.clone());
		files::create_leading_dirs(&path)?;
		let client = client.clone();
		let permit = Arc::clone(&sem).acquire_owned().await;
		let fut = async move {
			let response = client.get(url).send();
			let _permit = permit;
			tokio::fs::write(&path, response.await?.error_for_status()?.bytes().await?).await?;
			Ok::<(), AssetsError>(())
		};
		join.spawn(fut);
		num_done += 1;
		printer.print(&cformat!("(<b>{}</b><k!>/</k!><b>{}</b>) <k!>{}", num_done, count, key));
	}

	while let Some(asset) = join.join_next().await {
		match asset? {
			Ok(name) => name,
			Err(err) => {
				cprintln!("<r>Failed to download asset, skipping...\n{}", err);
				continue;
			}
		};
	}

	printer.print(&cformat!("<g>Assets downloaded."));
	printer.finish();

	Ok(out)
}

/// Gets the path to a stored game jar file
pub fn game_jar_path(kind: &InstKind, version: &str, paths: &Paths) -> PathBuf {
	let side_str = kind.to_string();
	paths.jars.join(format!("{version}_{side_str}.jar"))
}

/// Downloads the game jar file
pub async fn get_game_jar(
	kind: &InstKind,
	version_json: &json::JsonObject,
	version: &str,
	paths: &Paths,
	manager: &UpdateManager,
) -> Result<(), CreateError> {
	let side_str = kind.to_string();
	let path = game_jar_path(kind, version, paths);
	if !manager.should_update_file(&path) {
		return Ok(());
	}
	let mut printer = ReplPrinter::from_options(manager.print.clone());
	
	printer.print(&format!("Downloading {side_str} jar..."));
	let download =
		json::access_object(json::access_object(&version_json, "downloads")?, &side_str)?;
	let url = json::access_str(download, "url")?;
	tokio::fs::write(&path, download_bytes(url).await?).await?;
	printer.print(&cformat!("<g>{} jar downloaded.", cap_first_letter(&side_str)));
	
	Ok(())
}
