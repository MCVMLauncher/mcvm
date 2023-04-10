use crate::data::instance::Side;
use crate::data::profile::update::UpdateManager;
use crate::io::files::{self, paths::Paths};
use crate::io::java::classpath::Classpath;
use crate::util::json::{self, JsonObject, JsonType};
use crate::util::print::ReplPrinter;
use crate::util::{cap_first_letter, mojang, self};

use anyhow::{bail, Context};
use color_print::{cformat, cprintln};
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use zip::ZipArchive;

use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::download::{download_file, download_text, FD_SENSIBLE_LIMIT};

/// Obtain the raw version manifest contents
async fn get_version_manifest_contents(paths: &Paths) -> anyhow::Result<String> {
	let mut path = paths.internal.join("versions");
	files::create_dir_async(&path).await?;
	path.push("manifest.json");

	let text = download_text("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
		.await
		.context("Failed to download manifest")?;
	tokio::fs::write(&path, &text)
		.await
		.context("Failed to write manifest to a file")?;

	Ok(text)
}

/// Get the version manifest as a JSON object
pub async fn get_version_manifest(paths: &Paths) -> anyhow::Result<Box<json::JsonObject>> {
	let mut manifest_contents = get_version_manifest_contents(paths)
		.await
		.context("Failed to download manifest contents")?;
	let manifest = match json::parse_object(&manifest_contents) {
		Ok(manifest) => manifest,
		Err(..) => {
			cprintln!("<r>Failed to parse version manifest. Redownloading...");
			manifest_contents = get_version_manifest_contents(paths)
				.await
				.context("Failed to donwload manifest contents")?;
			json::parse_object(&manifest_contents)?
		}
	};
	Ok(manifest)
}

/// Make an ordered list of versions from the manifest to use for matching
pub fn make_version_list(version_manifest: &json::JsonObject) -> anyhow::Result<Vec<String>> {
	let versions = json::access_array(version_manifest, "versions")?;
	let mut out = Vec::new();
	for entry in versions {
		let entry_obj = json::ensure_type(entry.as_object(), JsonType::Obj)?;
		out.push(json::access_str(entry_obj, "id")?.to_owned());
	}
	out.reverse();
	Ok(out)
}

/// Gets the specific version info JSON file for a Minecraft version
pub async fn get_version_json(
	version: &str,
	version_manifest: &json::JsonObject,
	paths: &Paths,
) -> anyhow::Result<Box<json::JsonObject>> {
	let version_string = version.to_owned();

	let versions = json::access_array(version_manifest, "versions")?;
	let mut version_url: Option<&str> = None;
	for entry in versions.iter() {
		let entry = json::ensure_type(entry.as_object(), JsonType::Obj)?;
		if json::access_str(entry, "id")? == version_string {
			version_url = Some(json::access_str(entry, "url")?);
		}
	}
	if version_url.is_none() {
		bail!("Minecraft version does not exist or was not found in the manifest");
	}

	let version_json_name: String = version_string.clone() + ".json";
	let version_folder = paths.internal.join("versions").join(version_string);
	files::create_dir_async(&version_folder).await?;
	let text = download_text(version_url.expect("Version does not exist"))
		.await
		.context("Failed to download version JSON")?;
	tokio::fs::write(version_folder.join(version_json_name), &text)
		.await
		.context("Failed to write version JSON to a file")?;

	let version_doc = json::parse_object(&text).context("Failed to parse version JSON")?;

	Ok(version_doc)
}

/// Checks the rules of a game library to see if it should be installed
fn is_library_allowed(lib: &JsonObject) -> anyhow::Result<bool> {
	if let Some(rules) = lib.get("rules") {
		let rules = json::ensure_type(rules.as_array(), JsonType::Arr)?;
		for rule in rules.iter() {
			let rule = json::ensure_type(rule.as_object(), JsonType::Obj)?;
			let action = json::access_str(rule, "action")?;
			if let Some(os) = rule.get("os") {
				let os = json::ensure_type(os.as_object(), JsonType::Obj)?;
				let os_name = json::access_str(os, "name")?;
				let allowed = mojang::is_allowed(action);
				if allowed != (os_name == util::OS_STRING) {
					return Ok(false);
				}
			}
		}
	}
	Ok(true)
}

/// Extract the files of a native library into the natives directory
fn extract_native_library(path: &Path, natives_dir: &Path) -> anyhow::Result<()> {
	let file = File::open(path)?;
	let mut zip = ZipArchive::new(file)?;
	for i in 0..zip.len() {
		let mut file = zip.by_index(i)?;
		let rel_path = PathBuf::from(
			file.enclosed_name()
				.context("Invalid compressed file path")?,
		);
		if let Some(extension) = rel_path.extension() {
			match extension.to_str() {
				Some("so" | "dylib" | "dll") => {
					let mut out_file = File::create(natives_dir.join(rel_path))?;
					std::io::copy(&mut file, &mut out_file)
						.context("Failed to copy compressed file")?;
				}
				_ => continue,
			}
		}
	}

	Ok(())
}

/// Gets the list of allowed libraries from the version json
/// and also the number of libraries found.
pub fn get_lib_list(
	version_json: &json::JsonObject,
) -> anyhow::Result<impl Iterator<Item = &JsonObject>> {
	let libraries = json::access_array(version_json, "libraries")?;
	let libraries = libraries.iter().filter_map(|lib| {
		let lib = json::ensure_type(lib.as_object(), JsonType::Obj).ok()?;
		if !is_library_allowed(lib).ok()? {
			None
		} else {
			Some(lib)
		}
	});

	Ok(libraries)
}

/// Downloads base client libraries.
/// Returns a set of files to be added to the update manager.
pub async fn get_libraries(
	version_json: &json::JsonObject,
	paths: &Paths,
	version: &str,
	manager: &UpdateManager,
) -> anyhow::Result<HashSet<PathBuf>> {
	let mut files = HashSet::new();
	let libraries_path = paths.internal.join("libraries");
	files::create_dir_async(&libraries_path).await?;
	let natives_path = paths
		.internal
		.join("versions")
		.join(version)
		.join("natives");
	files::create_dir_async(&natives_path).await?;
	let natives_jars_path = paths.internal.join("natives");

	let mut native_paths = Vec::new();
		
	let libraries = get_lib_list(version_json)?;

	let mut libs_to_download = Vec::new();

	for lib in libraries {
		let name = json::access_str(lib, "name")?;
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives) = lib.get("natives") {
			let natives = json::ensure_type(natives.as_object(), JsonType::Obj)?;
			let key = json::access_str(natives, util::OS_STRING)?
				.replace("${arch}", util::TARGET_BITS_STR);
			let classifier =
				json::access_object(json::access_object(downloads, "classifiers")?, &key)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);

			native_paths.push((path.clone(), name.to_owned()));
			if !manager.should_update_file(&path) {
				continue;
			}
			libs_to_download.push((name, classifier.clone(), path));
			continue;
		}
		if let Some(artifact) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact.as_object(), JsonType::Obj)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			if !manager.should_update_file(&path) {
				continue;
			}
			libs_to_download.push((name, artifact.clone(), path));
			continue;
		}
	}

	let mut printer = ReplPrinter::from_options(manager.print.clone());

	if manager.print.verbose && libs_to_download.len() > 0 {
		cprintln!("Downloading <b>{}</> libraries...", libs_to_download.len());
	}

	let client = Client::new();
	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	for (name, library, path) in libs_to_download {
		printer.print(&cformat!("Downloading library <b!>{}</>...", name));
		files::create_leading_dirs_async(&path).await?;
		files.insert(path.clone());
		let url = json::access_str(&library, "url")?.to_owned();

		let client = client.clone();
		let permit = Arc::clone(&sem).acquire_owned().await;
		let fut = async move {
			let response = client.get(url).send();
			let _permit = permit;
			tokio::fs::write(&path, response.await?.error_for_status()?.bytes().await?).await?;
			Ok::<(), anyhow::Error>(())
		};
		join.spawn(fut);
	}

	while let Some(lib) = join.join_next().await {
		match lib? {
			Ok(name) => name,
			Err(err) => {
				cprintln!("<r>Failed to download asset, skipping...\n{}", err);
				continue;
			}
		};
	}

	for (path, name) in native_paths {
		printer.print(&cformat!("Extracting library <b!>{}...", name));
		extract_native_library(&path, &natives_path)
			.with_context(|| format!("Failed to extract native library {name}"))?;
	}

	printer.print(&cformat!("<g>Libraries downloaded."));
	printer.finish();

	Ok(files)
}

/// Gets the classpath from Minecraft libraries
pub fn get_lib_classpath(
	version_json: &json::JsonObject,
	paths: &Paths,
) -> anyhow::Result<Classpath> {
	let natives_jars_path = paths.internal.join("natives");
	let libraries_path = paths.internal.join("libraries");

	let mut classpath = Classpath::new();
	let libraries = get_lib_list(version_json).context("Failed to get list of libraries")?;
	for lib in libraries {
		let downloads = json::access_object(lib, "downloads")?;
		if let Some(natives) = lib.get("natives") {
			let natives = json::ensure_type(natives.as_object(), JsonType::Obj)?;
			let key = json::access_str(natives, util::OS_STRING)?
				.replace("${arch}", util::TARGET_BITS_STR);
			let classifier =
				json::access_object(json::access_object(downloads, "classifiers")?, &key)?;

			let path = natives_jars_path.join(json::access_str(classifier, "path")?);
			classpath.add_path(&path);

			continue;
		}
		if let Some(artifact) = downloads.get("artifact") {
			let artifact = json::ensure_type(artifact.as_object(), JsonType::Obj)?;
			let path = libraries_path.join(json::access_str(artifact, "path")?);
			classpath.add_path(&path);
			continue;
		}
	}
	Ok(classpath)
}

async fn download_asset_index(url: &str, path: &Path) -> anyhow::Result<Box<json::JsonObject>> {
	let text = download_text(url)
		.await
		.context("Failed to download index")?;
	tokio::fs::write(path, &text)
		.await
		.context("Failed to write index to a file")?;

	let doc = json::parse_object(&text).context("Failed to parse index")?;
	Ok(doc)
}

/// Download assets used by the client, such as game resources and icons.
pub async fn get_assets(
	version_json: &json::JsonObject,
	paths: &Paths,
	version: &str,
	manager: &UpdateManager,
) -> anyhow::Result<HashSet<PathBuf>> {
	let mut out = HashSet::new();
	let version_string = version.to_owned();
	let indexes_dir = paths.assets.join("indexes");
	files::create_dir_async(&indexes_dir).await?;

	let index_path = indexes_dir.join(version_string + ".json");
	let index_url = json::access_str(json::access_object(version_json, "assetIndex")?, "url")?;

	let objects_dir = paths.assets.join("objects");
	files::create_dir_async(&objects_dir).await?;
	// Apparently this directory name is used for older game versions
	let virtual_dir = paths.assets.join("virtual");
	if !manager.force && virtual_dir.exists() && !virtual_dir.is_symlink() {
		files::dir_symlink(&virtual_dir, &objects_dir)
			.context("Failed to symlink virtual assets")?;
	}

	let index = match download_asset_index(index_url, &index_path).await {
		Ok(val) => val,
		Err(err) => {
			cprintln!(
				"<r>Failed to obtain asset index:\n{}\nRedownloading...",
				err
			);
			download_asset_index(index_url, &index_path)
				.await
				.context("Failed to obtain asset index")?
		}
	};

	let assets = json::access_object(&index, "objects")?.clone();
	
	let mut assets_to_download = Vec::new();
	for (name, asset) in assets {
		let asset = json::ensure_type(asset.as_object(), JsonType::Obj)?;

		let hash = json::access_str(asset, "hash")?.to_owned();
		let hash_path = format!("{}/{hash}", hash[..2].to_owned());
		let url = format!("https://resources.download.minecraft.net/{hash_path}");

		let path = objects_dir.join(&hash_path);
		if !manager.should_update_file(&path) {
			continue;
		}

		out.insert(path.clone());
		files::create_leading_dirs_async(&path).await?;
		assets_to_download.push((name, url, path));
	}

	let mut printer = ReplPrinter::from_options(manager.print.clone());
	let count = assets_to_download.len();
	if manager.print.verbose && count > 0 {
		cprintln!("Downloading <b>{}</> assets...", count);
	}

	let mut num_done = 0;
	let client = Client::new();
	let mut join = JoinSet::new();
	// Used to limit the number of open file descriptors
	let sem = Arc::new(Semaphore::new(FD_SENSIBLE_LIMIT));
	for (name, url, path) in assets_to_download {
		let client = client.clone();
		let permit = Arc::clone(&sem).acquire_owned().await;
		let fut = async move {
			let response = client.get(url).send();
			let _permit = permit;
			tokio::fs::write(&path, response.await?.error_for_status()?.bytes().await?).await?;
			Ok::<(), anyhow::Error>(())
		};
		join.spawn(fut);
		num_done += 1;
		printer.print(&cformat!(
			"(<b>{}</b><k!>/</k!><b>{}</b>) <k!>{}",
			num_done,
			count,
			name
		));
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
pub fn game_jar_path(side: Side, version: &str, paths: &Paths) -> PathBuf {
	let side_str = side.to_string();
	paths.jars.join(format!("{version}_{side_str}.jar"))
}

/// Downloads the game jar file
pub async fn get_game_jar(
	side: Side,
	version_json: &json::JsonObject,
	version: &str,
	paths: &Paths,
	manager: &UpdateManager,
) -> anyhow::Result<()> {
	let side_str = side.to_string();
	let path = game_jar_path(side, version, paths);
	if !manager.should_update_file(&path) {
		return Ok(());
	}
	let mut printer = ReplPrinter::from_options(manager.print.clone());

	printer.print(&format!("Downloading {side_str} jar..."));
	let download = json::access_object(json::access_object(version_json, "downloads")?, &side_str)?;
	let url = json::access_str(download, "url")?;
	download_file(url, &path)
		.await
		.context("Failed to download file")?;
	printer.print(&cformat!(
		"<g>{} jar downloaded.",
		cap_first_letter(&side_str)
	));

	Ok(())
}
