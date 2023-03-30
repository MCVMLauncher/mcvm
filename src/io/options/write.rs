use std::io::Write;
use std::collections::HashMap;

use crate::util::{ToInt, versions::VersionPattern};

use super::read::{Options, FullscreenResolution};

/// Creates the string for the list of resource packs
fn write_resource_packs(resource_packs: &[String]) -> String {
	let names = resource_packs.iter().map(|x| format!("\"{x}\","));
	let mut names_joined = String::new();
	for name in names {
		names_joined.push_str(&name);
	}

	format!("[{names_joined}]")
}

/// Creates the string for fullscreen resolution
fn write_fullscreen_resolution(resolution: &FullscreenResolution) -> String {
	format!(
		"{}x{}@{}:{}",
		resolution.width.to_string(),
		resolution.height.to_string(),
		resolution.refresh_rate.to_string(),
		resolution.color_bits.to_string()
	)
}

/// Write options to a list of keys
pub fn write_keys(
	options: &Options,
	version: &str,
	versions: &[String],
) -> anyhow::Result<HashMap<String, String>> {
	let mut out = HashMap::new();
	let client = &options.client;

	// Version checks
	let after_12w50a = VersionPattern::After(String::from("12w50a")).matches_single(version, versions);
	let after_14w28a = VersionPattern::After(String::from("14w28a")).matches_single(version, versions);
	let after_17w06a = VersionPattern::After(String::from("17w06a")).matches_single(version, versions);
	let after_17w47a = VersionPattern::After(String::from("17w47a")).matches_single(version, versions);
	let after_18w15a = VersionPattern::After(String::from("18w15a")).matches_single(version, versions);
	let after_18w21a = VersionPattern::After(String::from("18w21a")).matches_single(version, versions);
	let after_1_13_pre2 = VersionPattern::After(String::from("1.13-pre2")).matches_single(version, versions);
	let after_1_15_2_pre1 = VersionPattern::After(String::from("1.15.2-pre1")).matches_single(version, versions);
	let after_1_16_4_rc1 = VersionPattern::After(String::from("1.16.4-rc1")).matches_single(version, versions);
	let after_21w13a = VersionPattern::After(String::from("21w13a")).matches_single(version, versions);
	let after_21w37a = VersionPattern::After(String::from("21w37a")).matches_single(version, versions);
	let after_21w38a = VersionPattern::After(String::from("21w38a")).matches_single(version, versions);
	let after_21w42a = VersionPattern::After(String::from("21w42a")).matches_single(version, versions);
	let after_1_18_pre2 = VersionPattern::After(String::from("1.18-pre2")).matches_single(version, versions);
	let after_1_18_2_pre1 = VersionPattern::After(String::from("1.18.2-pre1")).matches_single(version, versions);
	let after_22w11a = VersionPattern::After(String::from("22w11a")).matches_single(version, versions);
	let after_22w15a = VersionPattern::After(String::from("22w15a")).matches_single(version, versions);

	let before_15w31a = VersionPattern::Before(String::from("15w31a")).matches_single(version, versions);
	let before_1_19_4 = VersionPattern::Before(String::from("1.19.4")).matches_single(version, versions);

	// TODO: Add actual data version
	// out.insert(String::from("version"), client.data_version.to_string());
	out.insert(String::from("autoJump"), client.control.auto_jump.to_string());
	if after_17w47a {
		out.insert(String::from("autoSuggestions"), client.chat.auto_command_suggestions.to_string());
	}
	out.insert(String::from("chatColors"), client.chat.enable_colors.to_string());
	out.insert(String::from("chatLinks"), client.chat.enable_links.to_string());
	out.insert(String::from("chatLinksPrompt"), client.chat.prompt_links.to_string());
	out.insert(String::from("enableVsync"), client.video.vsync.to_string());
	out.insert(String::from("entityShadows"), client.video.entity_shadows.to_string());
	out.insert(String::from("forceUnicodeFont"), client.chat.force_unicode.to_string());
	out.insert(String::from("discrete_mouse_scroll"), client.control.discrete_mouse_scroll.to_string());
	out.insert(String::from("invertYMouse"), client.control.invert_mouse_y.to_string());
	out.insert(String::from("realmsNotifications"), client.realms_notifications.to_string());
	out.insert(String::from("reducedDebugInfo"), client.reduced_debug_info.to_string());
	out.insert(String::from("showSubtitles"), client.sound.show_subtitles.to_string());
	if after_22w11a {
		out.insert(String::from("directionalAudio"), client.sound.directional_audio.to_string());
	}
	out.insert(String::from("touchscreen"), client.control.enable_touchscreen.to_string());
	out.insert(String::from("bobView"), client.video.view_bobbing.to_string());
	out.insert(String::from("toggleCrouch"), client.control.toggle_crouch.to_string());
	out.insert(String::from("toggleSprint"), client.control.toggle_sprint.to_string());
	if after_21w13a {
		out.insert(String::from("darkMojangStudiosBackground"), client.video.dark_mojang_background.to_string());
	}
	if after_21w37a {
		out.insert(String::from("hideLightningFlashes"), client.video.hide_lightning_flashes.to_string());
		out.insert(String::from("prioritizeChunkUpdates"), client.video.chunk_updates_mode.to_int().to_string());
		if let Some(device) = &client.sound.device {
			out.insert(String::from("soundDevice"), device.clone());
		}
	}
	out.insert(String::from("mouseSensitivity"), client.control.mouse_sensitivity.to_string());
	out.insert(String::from("fov"), client.video.fov.to_string());
	out.insert(String::from("screenEffectScale"), client.video.screen_effect_scale.to_string());
	out.insert(String::from("fovEffectScale"), client.video.fov_effect_scale.to_string());
	if after_22w15a {
		out.insert(String::from("darknessEffectScale"), client.video.darkness_effect_scale.to_string());
	}
	out.insert(String::from("gamma"), client.video.brightness.to_string());
	out.insert(String::from("renderDistance"), client.video.render_distance.to_string());
	if after_21w38a {
		out.insert(String::from("simulationDistance"), client.video.simulation_distance.to_string());
	}
	out.insert(String::from("entityDistanceScaling"), client.video.entity_distance_scaling.to_string());
	out.insert(String::from("guiScale"), client.video.gui_scale.to_string());
	out.insert(String::from("particles"), client.video.particles.to_int().to_string());
	out.insert(String::from("maxFps"), client.video.max_fps.to_string());
	out.insert(String::from("difficulty"), client.difficulty.to_int().to_string());
	out.insert(String::from("graphicsMode"), client.video.graphics_mode.to_int().to_string());
	out.insert(String::from("ao"), client.video.smooth_lighting.to_string());
	if after_18w15a {
		out.insert(String::from("biomeBlendRadius"), client.video.biome_blend.to_string());
	}
	out.insert(String::from("renderClouds"), client.video.clouds.to_string());
	out.insert(String::from("resourcePacks"), write_resource_packs(&client.resource_packs));
	out.insert(String::from("incompatibleResourcePacks"), String::from("[]"));
	out.insert(String::from("lang"), client.language.clone());
	out.insert(String::from("chatVisibility"), client.chat.visibility.to_int().to_string());
	out.insert(String::from("chatOpacity"), client.chat.opacity.to_string());
	out.insert(String::from("chatLineSpacing"), client.chat.line_spacing.to_string());
	out.insert(String::from("textBackgroundOpacity"), client.chat.background_opacity.to_string());
	out.insert(String::from("backgroundForChatOnly"), client.chat.background_for_chat_only.to_string());
	out.insert(String::from("hideServerAddress"), client.hide_server_address.to_string());
	out.insert(String::from("advancedItemTooltips"), client.advanced_item_tooltips.to_string());
	out.insert(String::from("pauseOnLostFocus"), client.pause_on_lost_focus.to_string());
	out.insert(String::from("overrideWidth"), client.video.window_width.to_string());
	out.insert(String::from("overrideHeight"), client.video.window_height.to_string());
	if after_12w50a && before_1_19_4 {
		out.insert(String::from("heldItemTooltips"), client.held_item_tooltips.to_string());
	}
	out.insert(String::from("chatHeightFocused"), client.chat.focused_height.to_string());
	out.insert(String::from("chatDelay"), client.chat.delay.to_string());
	out.insert(String::from("chatHeightUnfocused"), client.chat.unfocused_height.to_string());
	out.insert(String::from("chatScale"), client.chat.scale.to_string());
	out.insert(String::from("chatWidth"), client.chat.width.to_string());
	out.insert(String::from("mipmapLevels"), client.video.mipmap_levels.to_string());
	out.insert(String::from("useNativeTransport"), client.use_native_transport.to_string());
	out.insert(String::from("mainHand"), client.main_hand.to_string());
	if after_17w06a {
		out.insert(String::from("narrator"), client.chat.narrator_mode.to_int().to_string());
		out.insert(String::from("tutorialStep"), client.tutorial_step.to_string());
	}
	if after_18w21a {
		out.insert(String::from("mouseWheelSensitivity"), client.control.mouse_wheel_sensitivity.to_string());
	}
	out.insert(String::from("rawMouseInput"), client.control.raw_mouse_input.to_string());
	if after_1_13_pre2 {
		out.insert(String::from("glDebugVerbosity"), client.log_level.to_int().to_string());
	}
	if after_1_15_2_pre1 {
		out.insert(String::from("skipMultiplayerWarning"), client.skip_multiplayer_warning.to_string());
	}
	if after_1_18_2_pre1 {
		out.insert(String::from("skipRealms32bitWarning"), client.skip_realms_32_bit_warning.to_string());
	}
	if after_1_16_4_rc1 {
		out.insert(String::from("hideMatchedNames"), client.hide_matched_names.to_string());
		out.insert(String::from("joinedFirstServer"), client.joined_server.to_string());
	}
	out.insert(String::from("hideBundleTutorial"), client.hide_bundle_tutorial.to_string());
	out.insert(String::from("syncChunkWrites"), client.sync_chunk_writes.to_string());
	if after_21w42a {
		out.insert(String::from("showAutosaveIndicator"), client.show_autosave_indicator.to_string());
	}
	if after_1_18_pre2 {
		out.insert(String::from("allowServerListing"), client.allow_server_listing.to_string());
	}
	// Keybinds
	out.insert(String::from("key_key.attack"), client.control.keys.attack.clone());
	out.insert(String::from("key_key.use"), client.control.keys.r#use.clone());
	out.insert(String::from("key_key.forward"), client.control.keys.forward.clone());
	out.insert(String::from("key_key.left"), client.control.keys.left.clone());
	out.insert(String::from("key_key.back"), client.control.keys.back.clone());
	out.insert(String::from("key_key.right"), client.control.keys.right.clone());
	out.insert(String::from("key_key.jump"), client.control.keys.jump.clone());
	out.insert(String::from("key_key.sneak"), client.control.keys.sneak.clone());
	out.insert(String::from("key_key.sprint"), client.control.keys.sprint.clone());
	out.insert(String::from("key_key.drop"), client.control.keys.drop.clone());
	out.insert(String::from("key_key.inventory"), client.control.keys.inventory.clone());
	out.insert(String::from("key_key.chat"), client.control.keys.chat.clone());
	out.insert(String::from("key_key.playerlist"), client.control.keys.playerlist.clone());
	out.insert(String::from("key_key.pickItem"), client.control.keys.pick_item.clone());
	out.insert(String::from("key_key.command"), client.control.keys.command.clone());
	out.insert(String::from("key_key.socialInteractions"), client.control.keys.social_interactions.clone());
	out.insert(String::from("key_key.screenshot"), client.control.keys.screenshot.clone());
	out.insert(String::from("key_key.togglePerspective"), client.control.keys.toggle_perspective.clone());
	out.insert(String::from("key_key.smoothCamera"), client.control.keys.smooth_camera.clone());
	out.insert(String::from("key_key.fullscreen"), client.control.keys.fullscreen.clone());
	out.insert(String::from("key_key.spectatorOutlines"), client.control.keys.spectator_outlines.clone());
	out.insert(String::from("key_key.swapOffhand"), client.control.keys.swap_offhand.clone());
	if after_17w06a {
		out.insert(String::from("key_key.saveToolbarActivator"), client.control.keys.save_toolbar.clone());
		out.insert(String::from("key_key.loadToolbarActivator"), client.control.keys.load_toolbar.clone());
		out.insert(String::from("key_key.advancements"), client.control.keys.advancements.clone());
	}
	out.insert(String::from("key_key.hotbar.1"), client.control.keys.hotbar_1.clone());
	out.insert(String::from("key_key.hotbar.2"), client.control.keys.hotbar_2.clone());
	out.insert(String::from("key_key.hotbar.3"), client.control.keys.hotbar_3.clone());
	out.insert(String::from("key_key.hotbar.4"), client.control.keys.hotbar_4.clone());
	out.insert(String::from("key_key.hotbar.5"), client.control.keys.hotbar_5.clone());
	out.insert(String::from("key_key.hotbar.6"), client.control.keys.hotbar_6.clone());
	out.insert(String::from("key_key.hotbar.7"), client.control.keys.hotbar_7.clone());
	out.insert(String::from("key_key.hotbar.8"), client.control.keys.hotbar_8.clone());
	out.insert(String::from("key_key.hotbar.9"), client.control.keys.hotbar_9.clone());
	// Volumes
	out.insert(String::from("soundCategory_master"), client.sound.volume.master.to_string());
	out.insert(String::from("soundCategory_music"), client.sound.volume.music.to_string());
	out.insert(String::from("soundCategory_record"), client.sound.volume.record.to_string());
	out.insert(String::from("soundCategory_weather"), client.sound.volume.weather.to_string());
	out.insert(String::from("soundCategory_block"), client.sound.volume.block.to_string());
	out.insert(String::from("soundCategory_hostile"), client.sound.volume.hostile.to_string());
	out.insert(String::from("soundCategory_neutral"), client.sound.volume.neutral.to_string());
	out.insert(String::from("soundCategory_player"), client.sound.volume.player.to_string());
	out.insert(String::from("soundCategory_ambient"), client.sound.volume.ambient.to_string());
	out.insert(String::from("soundCategory_voice"), client.sound.volume.voice.to_string());
	// Model parts
	out.insert(String::from("modelPart_cape"), client.skin.cape.to_string());
	out.insert(String::from("modelPart_jacket"), client.skin.jacket.to_string());
	out.insert(String::from("modelPart_left_sleeve"), client.skin.left_sleeve.to_string());
	out.insert(String::from("modelPart_right_sleeve"), client.skin.right_sleeve.to_string());
	out.insert(String::from("modelPart_left_pants_leg"), client.skin.left_pants.to_string());
	out.insert(String::from("modelPart_right_pants_leg"), client.skin.right_pants.to_string());
	out.insert(String::from("modelPart_hat"), client.skin.hat.to_string());
	if after_14w28a && before_15w31a {
		out.insert(String::from("allowBlockAlternatives"), client.video.allow_block_alternatives.to_string());
	}

	if let Some(resolution) = &client.video.fullscreen_resolution {
		out.insert(String::from("fullscreenResolution"), write_fullscreen_resolution(resolution));
	}

	Ok(out)
}

/// Write an options key to a writer
pub fn write_key<W: Write>(key: &str, value: &str, writer: &mut W) -> anyhow::Result<()> {
	writeln!(writer, "{key}:{value}")?;
	
	Ok(())
}

#[cfg(test)]
mod tests {
	use crate::io::options::read::parse_options_str;
	use super::*;

	#[test]
	fn test_write_keys() {
		let options = parse_options_str("{}").unwrap();
		let versions = [String::from("1.18"), String::from("1.19.3")];
		let keys = write_keys(&options, "1.19.3", &versions).unwrap();
		assert_eq!(*keys.get("version").unwrap(), options.client.data_version.to_string());
	}
}
