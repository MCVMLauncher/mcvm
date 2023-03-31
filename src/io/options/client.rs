use std::{fmt::Display, collections::HashMap};

use serde::Deserialize;

use crate::util::{mojang::TARGET_64_BIT, ToInt};

use super::read::OptionsEnum;

#[derive(Deserialize, Debug, Clone)]
pub struct KeyOptions {
	#[serde(default = "default_key_attack")]
	pub attack: String,
	#[serde(default = "default_key_use")]
	pub r#use: String,
	#[serde(default = "default_key_forward")]
	pub forward: String,
	#[serde(default = "default_key_left")]
	pub left: String,
	#[serde(default = "default_key_back")]
	pub back: String,
	#[serde(default = "default_key_right")]
	pub right: String,
	#[serde(default = "default_key_jump")]
	pub jump: String,
	#[serde(default = "default_key_sneak")]
	pub sneak: String,
	#[serde(default = "default_key_sprint")]
	pub sprint: String,
	#[serde(default = "default_key_drop")]
	pub drop: String,
	#[serde(default = "default_key_inventory")]
	pub inventory: String,
	#[serde(default = "default_key_chat")]
	pub chat: String,
	#[serde(default = "default_key_playerlist")]
	pub playerlist: String,
	#[serde(default = "default_key_pick_item")]
	pub pick_item: String,
	#[serde(default = "default_key_command")]
	pub command: String,
	#[serde(default = "default_key_social_interactions")]
	pub social_interactions: String,
	#[serde(default = "default_key_screenshot")]
	pub screenshot: String,
	#[serde(default = "default_key_toggle_perspective")]
	pub toggle_perspective: String,
	#[serde(default = "default_key_smooth_camera")]
	pub smooth_camera: String,
	#[serde(default = "default_key_fullscreen")]
	pub fullscreen: String,
	#[serde(default = "default_key_spectator_outlines")]
	pub spectator_outlines: String,
	#[serde(default = "default_key_swap_offhand")]
	pub swap_offhand: String,
	#[serde(default = "default_key_save_toolbar")]
	pub save_toolbar: String,
	#[serde(default = "default_key_load_toolbar")]
	pub load_toolbar: String,
	#[serde(default = "default_key_advancements")]
	pub advancements: String,
	#[serde(default = "default_key_hotbar_1")]
	pub hotbar_1: String,
	#[serde(default = "default_key_hotbar_2")]
	pub hotbar_2: String,
	#[serde(default = "default_key_hotbar_3")]
	pub hotbar_3: String,
	#[serde(default = "default_key_hotbar_4")]
	pub hotbar_4: String,
	#[serde(default = "default_key_hotbar_5")]
	pub hotbar_5: String,
	#[serde(default = "default_key_hotbar_6")]
	pub hotbar_6: String,
	#[serde(default = "default_key_hotbar_7")]
	pub hotbar_7: String,
	#[serde(default = "default_key_hotbar_8")]
	pub hotbar_8: String,
	#[serde(default = "default_key_hotbar_9")]
	pub hotbar_9: String,
}

impl Default for KeyOptions {
	fn default() -> Self {
		Self {
			attack: default_key_attack(),
			r#use: default_key_use(),
			forward: default_key_forward(),
			left: default_key_left(),
			back: default_key_back(),
			right: default_key_right(),
			jump: default_key_jump(),
			sneak: default_key_sneak(),
			sprint: default_key_sprint(),
			drop: default_key_drop(),
			inventory: default_key_inventory(),
			chat: default_key_chat(),
			playerlist: default_key_playerlist(),
			pick_item: default_key_pick_item(),
			command: default_key_command(),
			social_interactions: default_key_social_interactions(),
			screenshot: default_key_screenshot(),
			toggle_perspective: default_key_toggle_perspective(),
			smooth_camera: default_key_smooth_camera(),
			fullscreen: default_key_fullscreen(),
			spectator_outlines: default_key_spectator_outlines(),
			swap_offhand: default_key_swap_offhand(),
			save_toolbar: default_key_save_toolbar(),
			load_toolbar: default_key_load_toolbar(),
			advancements: default_key_advancements(),
			hotbar_1: default_key_hotbar_1(),
			hotbar_2: default_key_hotbar_2(),
			hotbar_3: default_key_hotbar_3(),
			hotbar_4: default_key_hotbar_4(),
			hotbar_5: default_key_hotbar_5(),
			hotbar_6: default_key_hotbar_6(),
			hotbar_7: default_key_hotbar_7(),
			hotbar_8: default_key_hotbar_8(),
			hotbar_9: default_key_hotbar_9(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ControlOptions {
	#[serde(default)]
	pub keys: KeyOptions,
	#[serde(default = "default_auto_jump")]
	pub auto_jump: bool,
	#[serde(default = "default_discrete_mouse_scroll")]
	pub discrete_mouse_scroll: bool,
	#[serde(default = "default_invert_mouse_y")]
	pub invert_mouse_y: bool,
	#[serde(default = "default_enable_touchscreen")]
	pub enable_touchscreen: bool,
	#[serde(default = "default_toggle_sprint")]
	pub toggle_sprint: bool,
	#[serde(default = "default_toggle_crouch")]
	pub toggle_crouch: bool,
	#[serde(default = "default_mouse_sensitivity")]
	pub mouse_sensitivity: f32,
	#[serde(default = "default_mouse_wheel_sensitivity")]
	pub mouse_wheel_sensitivity: f32,
	#[serde(default = "default_raw_mouse_input")]
	pub raw_mouse_input: bool,
}

impl Default for ControlOptions {
	fn default() -> Self {
		Self {
			keys: KeyOptions::default(),
			auto_jump: default_auto_jump(),
			discrete_mouse_scroll: default_discrete_mouse_scroll(),
			invert_mouse_y: default_invert_mouse_y(),
			enable_touchscreen: default_enable_touchscreen(),
			toggle_sprint: default_toggle_sprint(),
			toggle_crouch: default_toggle_crouch(),
			mouse_sensitivity: default_mouse_sensitivity(),
			mouse_wheel_sensitivity: default_mouse_wheel_sensitivity(),
			raw_mouse_input: default_raw_mouse_input(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChatOptions {
	#[serde(default = "default_auto_command_suggestions")]
	pub auto_command_suggestions: bool,
	#[serde(default = "default_enable_chat_colors")]
	pub enable_colors: bool,
	#[serde(default = "default_enable_chat_links")]
	pub enable_links: bool,
	#[serde(default = "default_prompt_links")]
	pub prompt_links: bool,
	#[serde(default = "default_force_unicode")]
	pub force_unicode: bool,
	#[serde(default = "default_chat_visibility")]
	pub visibility: OptionsEnum<ChatVisibility>,
	#[serde(default = "default_chat_opacity")]
	pub opacity: f32,
	#[serde(default = "default_chat_line_spacing")]
	pub line_spacing: f32,
	#[serde(default = "default_text_background_opacity")]
	pub background_opacity: f32,
	#[serde(default = "default_background_for_chat_only")]
	pub background_for_chat_only: bool,
	#[serde(default = "default_chat_focused_height")]
	pub focused_height: f32,
	#[serde(default = "default_chat_unfocused_height")]
	pub unfocused_height: f32,
	#[serde(default = "default_chat_delay")]
	pub delay: f32,
	#[serde(default = "default_chat_scale")]
	pub scale: f32,
	#[serde(default = "default_chat_width")]
	pub width: f32,
	#[serde(default = "default_narrator_mode")]
	pub narrator_mode: OptionsEnum<NarratorMode>,
}

impl Default for ChatOptions {
	fn default() -> Self {
		Self {
			auto_command_suggestions: default_auto_command_suggestions(),
			enable_colors: default_enable_chat_colors(),
			enable_links: default_enable_chat_links(),
			prompt_links: default_prompt_links(),
			force_unicode: default_force_unicode(),
			visibility: default_chat_visibility(),
			opacity: default_chat_opacity(),
			line_spacing: default_chat_line_spacing(),
			background_opacity: default_text_background_opacity(),
			background_for_chat_only: default_background_for_chat_only(),
			focused_height: default_chat_focused_height(),
			unfocused_height: default_chat_unfocused_height(),
			delay: default_chat_delay(),
			scale: default_chat_scale(),
			width: default_chat_width(),
			narrator_mode: default_narrator_mode(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct VideoOptions {
	#[serde(default = "default_vsync")]
	pub vsync: bool,
	#[serde(default = "default_entity_shadows")]
	pub entity_shadows: bool,
	#[serde(default = "default_fullscreen")]
	pub fullscreen: bool,
	#[serde(default = "default_view_bobbing")]
	pub view_bobbing: bool,
	#[serde(default = "default_dark_mojang_background")]
	pub dark_mojang_background: bool,
	#[serde(default = "default_hide_lightning_flashes")]
	pub hide_lightning_flashes: bool,
	#[serde(default = "default_fov")]
	pub fov: u8,
	#[serde(default = "default_screen_effect_scale")]
	pub screen_effect_scale: f32,
	#[serde(default = "default_fov_effect_scale")]
	pub fov_effect_scale: f32,
	#[serde(default = "default_darkness_effect_scale")]
	pub darkness_effect_scale: f32,
	#[serde(default = "default_brightness")]
	pub brightness: f32,
	#[serde(default = "default_render_distance")]
	pub render_distance: u8,
	#[serde(default = "default_simulation_distance")]
	pub simulation_distance: u8,
	#[serde(default = "default_entity_distance_scaling")]
	pub entity_distance_scaling: f32,
	#[serde(default = "default_gui_scale")]
	pub gui_scale: u8,
	#[serde(default = "default_particles")]
	pub particles: OptionsEnum<ParticlesMode>,
	#[serde(default = "default_max_fps")]
	pub max_fps: u8,
	#[serde(default = "default_graphics_mode")]
	pub graphics_mode: OptionsEnum<GraphicsMode>,
	#[serde(default = "default_smooth_lighting")]
	pub smooth_lighting: bool,
	#[serde(default = "default_chunk_updates_mode")]
	pub chunk_updates_mode: OptionsEnum<ChunkUpdatesMode>,
	#[serde(default = "default_biome_blend")]
	pub biome_blend: u8,
	#[serde(default = "default_clouds")]
	pub clouds: CloudRenderMode,
	#[serde(default = "default_mipmap_levels")]
	pub mipmap_levels: u8,
	#[serde(default = "default_window_width")]
	pub window_width: u16,
	#[serde(default = "default_window_height")]
	pub window_height: u16,
	#[serde(default = "default_attack_indicator")]
	pub attack_indicator: OptionsEnum<AttackIndicatorMode>,
	#[serde(default = "default_fullscreen_resolution")]
	pub fullscreen_resolution: Option<FullscreenResolution>,
	#[serde(default = "default_allow_block_alternatives")]
	pub allow_block_alternatives: bool,
}

impl Default for VideoOptions {
	fn default() -> Self {
		Self {
			vsync: default_vsync(),
			entity_shadows: default_entity_shadows(),
			fullscreen: default_fullscreen(),
			view_bobbing: default_view_bobbing(),
			dark_mojang_background: default_dark_mojang_background(),
			hide_lightning_flashes: default_hide_lightning_flashes(),
			fov: default_fov(),
			screen_effect_scale: default_screen_effect_scale(),
			fov_effect_scale: default_fov_effect_scale(),
			darkness_effect_scale: default_darkness_effect_scale(),
			brightness: default_brightness(),
			render_distance: default_render_distance(),
			simulation_distance: default_simulation_distance(),
			entity_distance_scaling: default_entity_distance_scaling(),
			gui_scale: default_gui_scale(),
			particles: default_particles(),
			max_fps: default_max_fps(),
			graphics_mode: default_graphics_mode(),
			smooth_lighting: default_smooth_lighting(),
			chunk_updates_mode: default_chunk_updates_mode(),
			biome_blend: default_biome_blend(),
			clouds: default_clouds(),
			mipmap_levels: default_mipmap_levels(),
			window_width: default_window_width(),
			window_height: default_window_height(),
			attack_indicator: default_attack_indicator(),
			fullscreen_resolution: default_fullscreen_resolution(),
			allow_block_alternatives: default_allow_block_alternatives(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct VolumeOptions {
	#[serde(default = "default_sound_volume")]
	pub master: f32,
	#[serde(default = "default_sound_volume")]
	pub music: f32,
	#[serde(default = "default_sound_volume")]
	pub record: f32,
	#[serde(default = "default_sound_volume")]
	pub weather: f32,
	#[serde(default = "default_sound_volume")]
	pub block: f32,
	#[serde(default = "default_sound_volume")]
	pub hostile: f32,
	#[serde(default = "default_sound_volume")]
	pub neutral: f32,
	#[serde(default = "default_sound_volume")]
	pub player: f32,
	#[serde(default = "default_sound_volume")]
	pub ambient: f32,
	#[serde(default = "default_sound_volume")]
	pub voice: f32,
}

impl Default for VolumeOptions {
	fn default() -> Self {
		Self {
			master: default_sound_volume(),
			music: default_sound_volume(),
			record: default_sound_volume(),
			weather: default_sound_volume(),
			block: default_sound_volume(),
			hostile: default_sound_volume(),
			neutral: default_sound_volume(),
			player: default_sound_volume(),
			ambient: default_sound_volume(),
			voice: default_sound_volume(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct SoundOptions {
	#[serde(default)]
	pub volume: VolumeOptions,
	#[serde(default = "default_show_subtitles")]
	pub show_subtitles: bool,
	#[serde(default = "default_directional_audio")]
	pub directional_audio: bool,
	#[serde(default = "default_sound_device")]
	pub device: Option<String>,
}

impl Default for SoundOptions {
	fn default() -> Self {
		Self {
			volume: VolumeOptions::default(),
			show_subtitles: default_show_subtitles(),
			directional_audio: default_directional_audio(),
			device: default_sound_device(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct SkinOptions {
	#[serde(default = "default_skin_part")]
	pub cape: bool,
	#[serde(default = "default_skin_part")]
	pub jacket: bool,
	#[serde(default = "default_skin_part")]
	pub left_sleeve: bool,
	#[serde(default = "default_skin_part")]
	pub right_sleeve: bool,
	#[serde(default = "default_skin_part")]
	pub left_pants: bool,
	#[serde(default = "default_skin_part")]
	pub right_pants: bool,
	#[serde(default = "default_skin_part")]
	pub hat: bool,
}

impl Default for SkinOptions {
	fn default() -> Self {
		Self {
			cape: default_skin_part(),
			jacket: default_skin_part(),
			left_sleeve: default_skin_part(),
			right_sleeve: default_skin_part(),
			left_pants: default_skin_part(),
			right_pants: default_skin_part(),
			hat: default_skin_part(),
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct ClientOptions {
	#[serde(default = "default_data_version")]
	pub data_version: i16,
	#[serde(default)]
	pub video: VideoOptions,
	#[serde(default)]
	pub control: ControlOptions,
	#[serde(default)]
	pub chat: ChatOptions,
	#[serde(default)]
	pub sound: SoundOptions,
	#[serde(default)]
	pub skin: SkinOptions,
	#[serde(default)]
	pub custom: HashMap<String, String>,
	#[serde(default = "default_realms_notifications")]
	pub realms_notifications: bool,
	#[serde(default = "default_reduced_debug_info")]
	pub reduced_debug_info: bool,
	#[serde(default = "default_difficulty")]
	pub difficulty: OptionsEnum<Difficulty>,
	#[serde(default = "default_resource_packs")]
	pub resource_packs: Vec<String>,
	#[serde(default = "default_language")]
	pub language: String,
	#[serde(default = "default_tutorial_step")]
	pub tutorial_step: TutorialStep,
	#[serde(default = "default_skip_multiplayer_warning")]
	pub skip_multiplayer_warning: bool,
	#[serde(default = "default_skip_realms_32_bit_warning")]
	pub skip_realms_32_bit_warning: bool,
	#[serde(default = "default_hide_bundle_tutorial")]
	pub hide_bundle_tutorial: bool,
	#[serde(default = "default_joined_server")]
	pub joined_server: bool,
	#[serde(default = "default_sync_chunk_writes")]
	pub sync_chunk_writes: bool,
	#[serde(default = "default_use_native_transport")]
	pub use_native_transport: bool,
	#[serde(default = "default_held_item_tooltips")]
	pub held_item_tooltips: bool,
	#[serde(default = "default_advanced_item_tooltips")]
	pub advanced_item_tooltips: bool,
	#[serde(default = "default_log_level")]
	pub log_level: OptionsEnum<LogLevel>,
	#[serde(default = "default_hide_matched_names")]
	pub hide_matched_names: bool,
	#[serde(default = "default_pause_on_lost_focus")]
	pub pause_on_lost_focus: bool,
	#[serde(default = "default_main_hand")]
	pub main_hand: MainHand,
	#[serde(default = "default_hide_server_address")]
	pub hide_server_address: bool,
	#[serde(default = "default_show_autosave_indicator")]
	pub show_autosave_indicator: bool,
	#[serde(default = "default_allow_server_listing")]
	pub allow_server_listing: bool,
}

impl Default for ClientOptions {
	fn default() -> Self {
		Self {
			data_version: default_data_version(),
			video: VideoOptions::default(),
			control: ControlOptions::default(),
			chat: ChatOptions::default(),
			sound: SoundOptions::default(),
			skin: SkinOptions::default(),
			custom: HashMap::default(),
			realms_notifications: default_realms_notifications(),
			reduced_debug_info: default_reduced_debug_info(),
			difficulty: default_difficulty(),
			resource_packs: default_resource_packs(),
			language: default_language(),
			tutorial_step: default_tutorial_step(),
			skip_multiplayer_warning: default_skip_multiplayer_warning(),
			skip_realms_32_bit_warning: default_skip_realms_32_bit_warning(),
			hide_bundle_tutorial: default_hide_bundle_tutorial(),
			joined_server: default_joined_server(),
			sync_chunk_writes: default_sync_chunk_writes(),
			use_native_transport: default_use_native_transport(),
			held_item_tooltips: default_held_item_tooltips(),
			advanced_item_tooltips: default_advanced_item_tooltips(),
			log_level: default_log_level(),
			hide_matched_names: default_hide_matched_names(),
			pause_on_lost_focus: default_pause_on_lost_focus(),
			main_hand: default_main_hand(),
			hide_server_address: default_hide_server_address(),
			show_autosave_indicator: default_show_autosave_indicator(),
			allow_server_listing: default_allow_server_listing(),
		}
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsMode {
	Fast,
	Fancy,
	Fabulous,
}

impl ToInt for GraphicsMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ParticlesMode {
	All,
	Decreased,
	Minimal,
}

impl ToInt for ParticlesMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
	Peaceful,
	Easy,
	Normal,
	Hard,
}

impl ToInt for Difficulty {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ChunkUpdatesMode {
	Threaded,
	SemiBlocking,
	FullyBlocking,
}

impl ToInt for ChunkUpdatesMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CloudRenderMode {
	Fancy,
	Off,
	Fast,
}

impl Display for CloudRenderMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Fancy => "true",
			Self::Off => "false",
			Self::Fast => "fast",
		})
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ChatVisibility {
	Shown,
	CommandsOnly,
	Hidden,
}

impl ToInt for ChatVisibility {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MainHand {
	Left,
	Right,
}

impl Display for MainHand {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Left => "left",
			Self::Right => "right",
		})
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AttackIndicatorMode {
	Off,
	Crosshair,
	Hotbar,
}

impl ToInt for AttackIndicatorMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum NarratorMode {
	Off,
	All,
	Chat,
	System,
}

impl ToInt for NarratorMode {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TutorialStep {
	Movement,
	FindTree,
	PunchTree,
	OpenInventory,
	CraftPlanks,
	None,
}

impl Display for TutorialStep {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Movement => "movement",
			Self::FindTree => "find_tree",
			Self::PunchTree => "punch_tree",
			Self::OpenInventory => "open_inventory",
			Self::CraftPlanks => "craft_planks",
			Self::None => "none",
		})
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
	None,
	High,
	Medium,
	Low,
	Notification,
}

impl ToInt for LogLevel {
	fn to_int(&self) -> i32 {
		self.clone() as i32
	}
}

#[derive(Deserialize, Debug, Clone)]
pub struct FullscreenResolution {
	pub width: u32,
	pub height: u32,
	pub refresh_rate: u32,
	pub color_bits: u32,
}

fn default_data_version() -> i16 { 3337 }
fn default_auto_jump() -> bool { true }
fn default_auto_command_suggestions() -> bool { true }
fn default_enable_chat_colors() -> bool { true }
fn default_enable_chat_links() -> bool { true }
fn default_prompt_links() -> bool { true }
fn default_vsync() -> bool { true }
fn default_entity_shadows() -> bool { true }
fn default_force_unicode() -> bool { false }
fn default_discrete_mouse_scroll() -> bool { false }
fn default_invert_mouse_y() -> bool { false }
fn default_realms_notifications() -> bool { true }
fn default_reduced_debug_info() -> bool { false }
fn default_show_subtitles() -> bool { false }
fn default_directional_audio() -> bool { false }
fn default_enable_touchscreen() -> bool { false }
fn default_fullscreen() -> bool { false }
fn default_view_bobbing() -> bool { true }
fn default_toggle_sprint() -> bool { false }
fn default_toggle_crouch() -> bool { false }
fn default_dark_mojang_background() -> bool { false }
fn default_hide_lightning_flashes() -> bool { false }
fn default_mouse_sensitivity() -> f32 { 0.5 }
fn default_fov() -> u8 { 0 }
fn default_screen_effect_scale() -> f32 { 1.0 }
fn default_fov_effect_scale() -> f32 { 1.0 }
fn default_darkness_effect_scale() -> f32 { 1.0 }
fn default_brightness() -> f32 { 0.5 }
fn default_render_distance() -> u8 {
	if TARGET_64_BIT {
		12
	} else {
		8
	}
}
fn default_simulation_distance() -> u8 { default_render_distance() }
fn default_entity_distance_scaling() -> f32 { 1.0 }
fn default_gui_scale() -> u8 { 0 }
fn default_particles() -> OptionsEnum<ParticlesMode> { OptionsEnum::Mode(ParticlesMode::All) }
fn default_max_fps() -> u8 { 120 }
fn default_difficulty() -> OptionsEnum<Difficulty> { OptionsEnum::Mode(Difficulty::Normal) }
fn default_graphics_mode() -> OptionsEnum<GraphicsMode> { OptionsEnum::Mode(GraphicsMode::Fancy) }
fn default_smooth_lighting() -> bool { true }
fn default_chunk_updates_mode() -> OptionsEnum<ChunkUpdatesMode> { OptionsEnum::Mode(ChunkUpdatesMode::Threaded) }
fn default_biome_blend() -> u8 { 2 }
fn default_clouds() -> CloudRenderMode { CloudRenderMode::Fancy }
fn default_resource_packs() -> Vec<String> { vec![] }
fn default_language() -> String { String::from("en_us") }
fn default_sound_device() -> Option<String> { None }
fn default_chat_visibility() -> OptionsEnum<ChatVisibility> { OptionsEnum::Mode(ChatVisibility::Shown) }
fn default_chat_opacity() -> f32 { 1.0 }
fn default_chat_line_spacing() -> f32 { 0.0 }
fn default_text_background_opacity() -> f32 { 0.5 }
fn default_background_for_chat_only() -> bool { true }
fn default_hide_server_address() -> bool { false }
fn default_advanced_item_tooltips() -> bool { false }
fn default_pause_on_lost_focus() -> bool { false }
fn default_window_width() -> u16 { 0 }
fn default_window_height() -> u16 { 0 }
fn default_held_item_tooltips() -> bool { true }
fn default_chat_focused_height() -> f32 { 1.0 }
fn default_chat_unfocused_height() -> f32 { 0.4375 }
fn default_chat_delay() -> f32 { 0.0 }
fn default_chat_scale() -> f32 { 1.0 }
fn default_chat_width() -> f32 { 1.0 }
fn default_mipmap_levels() -> u8 { 4 }
fn default_use_native_transport() -> bool { true }
fn default_main_hand() -> MainHand { MainHand::Right }
fn default_attack_indicator() -> OptionsEnum<AttackIndicatorMode> { OptionsEnum::Mode(AttackIndicatorMode::Crosshair) }
fn default_narrator_mode() -> OptionsEnum<NarratorMode> { OptionsEnum::Mode(NarratorMode::Off) }
fn default_tutorial_step() -> TutorialStep { TutorialStep::None }
fn default_mouse_wheel_sensitivity() -> f32 { 1.0 }
fn default_raw_mouse_input() -> bool { true }
fn default_log_level() -> OptionsEnum<LogLevel> { OptionsEnum::Mode(LogLevel::High) }
fn default_skip_multiplayer_warning() -> bool { true }
fn default_skip_realms_32_bit_warning() -> bool { true }
fn default_hide_matched_names() -> bool { true }
fn default_joined_server() -> bool { true }
fn default_hide_bundle_tutorial() -> bool { true }
fn default_sync_chunk_writes() -> bool {
	if cfg!(target_os = "windows") {
		false
	} else {
		true
	}
}
fn default_show_autosave_indicator() -> bool { true }
fn default_allow_server_listing() -> bool { true }
fn default_sound_volume() -> f32 { 1.0 }
fn default_fullscreen_resolution() -> Option<FullscreenResolution> { None }
fn default_key_attack() -> String { String::from("key.mouse.left") }
fn default_key_use() -> String { String::from("key.mouse.right") }
fn default_key_forward() -> String { String::from("key.keyboard.w") }
fn default_key_left() -> String { String::from("key.keyboard.a") }
fn default_key_back() -> String { String::from("key.keyboard.s") }
fn default_key_right() -> String { String::from("key.keyboard.d") }
fn default_key_jump() -> String { String::from("key.keyboard.space") }
fn default_key_sneak() -> String { String::from("key.keyboard.left.control") }
fn default_key_sprint() -> String { String::from("key.keyboard.left.shift") }
fn default_key_drop() -> String { String::from("key.keyboard.q") }
fn default_key_inventory() -> String { String::from("key.keyboard.e") }
fn default_key_chat() -> String { String::from("key.keyboard.t") }
fn default_key_playerlist() -> String { String::from("key.keyboard.tab") }
fn default_key_pick_item() -> String { String::from("key.mouse.middle") }
fn default_key_command() -> String { String::from("key.keyboard.slash") }
fn default_key_social_interactions() -> String { String::from("key.keyboard.p") }
fn default_key_screenshot() -> String { String::from("key.keyboard.f2") }
fn default_key_toggle_perspective() -> String { String::from("key.keyboard.f5") }
fn default_key_smooth_camera() -> String { String::from("key.keyboard.unknown") }
fn default_key_fullscreen() -> String { String::from("key.keyboard.f11") }
fn default_key_spectator_outlines() -> String { String::from("key.keyboard.unknown") }
fn default_key_swap_offhand() -> String { String::from("key.keyboard.f") }
fn default_key_save_toolbar() -> String { String::from("key.keyboard.c") }
fn default_key_load_toolbar() -> String { String::from("key.keyboard.x") }
fn default_key_advancements() -> String { String::from("key.keyboard.l") }
fn default_key_hotbar_1() -> String { String::from("key.keyboard.1") }
fn default_key_hotbar_2() -> String { String::from("key.keyboard.2") }
fn default_key_hotbar_3() -> String { String::from("key.keyboard.3") }
fn default_key_hotbar_4() -> String { String::from("key.keyboard.4") }
fn default_key_hotbar_5() -> String { String::from("key.keyboard.5") }
fn default_key_hotbar_6() -> String { String::from("key.keyboard.6") }
fn default_key_hotbar_7() -> String { String::from("key.keyboard.7") }
fn default_key_hotbar_8() -> String { String::from("key.keyboard.8") }
fn default_key_hotbar_9() -> String { String::from("key.keyboard.9") }
fn default_skin_part() -> bool { true }
fn default_allow_block_alternatives() -> bool { true }