pub mod assets;
pub mod libraries;
pub mod version_manifest;

use crate::data::profile::update::UpdateManager;
use crate::io::files::paths::Paths;
use crate::util::cap_first_letter;
use crate::util::json;
use crate::util::print::ReplPrinter;
use mcvm_shared::instance::Side;

use anyhow::Context;
use color_print::cformat;
use reqwest::Client;

use super::download;

pub mod game_jar {
	use super::*;

	/// Downloads the game jar file
	pub async fn get(
		side: Side,
		client_json: &json::JsonObject,
		version: &str,
		paths: &Paths,
		manager: &UpdateManager,
	) -> anyhow::Result<()> {
		let side_str = side.to_string();
		let path = crate::io::minecraft::game_jar::get_path(side, version, paths);
		if !manager.should_update_file(&path) {
			return Ok(());
		}
		let mut printer = ReplPrinter::from_options(manager.print.clone());

		printer.print(&format!("Downloading {side_str} jar..."));
		let download =
			json::access_object(json::access_object(client_json, "downloads")?, &side_str)?;
		let url = json::access_str(download, "url")?;
		download::file(url, &path, &Client::new())
			.await
			.context("Failed to download file")?;
		let side_str = cap_first_letter(&side_str);
		printer.print(&cformat!("<g>{} jar downloaded.", side_str));

		Ok(())
	}
}
