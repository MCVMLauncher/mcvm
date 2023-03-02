pub mod preferences;

use preferences::ConfigPreferences;
use super::user::{User, UserKind, AuthState, Auth};
use super::profile::{Profile, InstanceRegistry};
use super::instance::{Instance, InstKind};
use crate::package::PkgConfig;
use crate::package::reg::{PkgRegistry, PkgRequest, PkgIdentifier};
use crate::util::versions::{VersionPattern, MinecraftVersion};
use crate::util::json::{self, JsonType};

use color_print::cprintln;
use serde_json::json;

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;

// Default program configuration
fn default_config() -> serde_json::Value {
	json!(
		{
			"users": {
				"example": {
					"type": "microsoft",
					"name": "ExampleUser441"
				}
			},
			"profiles": {
				"example": {
					"version": "1.19.3",
					"instances": {
						"example-client": {
							"type": "client"
						},
						"example-server": {
							"type": "server"
						}
					}
				}
			},
			"packages": []
		}
	)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	#[error("{}", .0)]
	File(#[from] std::io::Error),
	#[error("Failed to parse json:\n{}", .0)]
	Json(#[from] json::JsonError),
	#[error("Json operation failed:\n{}", .0)]
	SerdeJson(#[from] serde_json::Error),
	#[error("Invalid config content:\n{}", .0)]
	Content(#[from] ContentError)
}

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
	#[error("Unknown type {} for user {}", .0, .1)]
	UserType(String, String),
	#[error("Unknown type {} for instance {}", .0, .1)]
	InstType(String, String),
	#[error("Unknown type {} for package {}", .0, .1)]
	PkgType(String, String),
	#[error("Unknown default user '{}'", .0)]
	DefaultUserNotFound(String),
	#[error("Duplicate instance '{}'", .0)]
	DuplicateInstance(String),
	#[error("Package '{}': Local packages must specify their exact version without special patterns", .0)]
	LocalPackageVersion(String)
}

#[derive(Debug)]
pub struct Config {
	pub auth: Auth,
	pub instances: InstanceRegistry,
	pub profiles: HashMap<String, Box<Profile>>,
	pub packages: PkgRegistry,
	pub prefs: ConfigPreferences
}

impl Config {
	fn open(path: &PathBuf) -> Result<Box<json::JsonObject>, ConfigError> {
		if path.exists() {
			let doc = json::parse_object(&fs::read_to_string(path)?)?;
			Ok(doc)
		} else {
			let doc = default_config();
			fs::write(path, serde_json::to_string_pretty(&doc)?)?;
			Ok(Box::new(json::ensure_type(doc.as_object(), JsonType::Obj)?.clone()))
		}
	}
	
	fn load_from_obj(obj: &json::JsonObject) -> Result<Self, ConfigError> {
		let mut auth = Auth::new();
		let mut instances = InstanceRegistry::new();
		let mut profiles = HashMap::new();
		// Preferences
		let (prefs, repositories) = ConfigPreferences::read(obj.get("preferences"))?;

		let mut packages = PkgRegistry::new(repositories);

		// Users
		let users = json::access_object(obj, "users")?;
		for (user_id, user_val) in users.iter() {
			let user_obj = json::ensure_type(user_val.as_object(), JsonType::Obj)?;
			let kind = match json::access_str(user_obj, "type")? {
				"microsoft" => Ok(UserKind::Microsoft),
				"demo" => Ok(UserKind::Demo),
				typ => Err(ContentError::UserType(typ.to_string(), user_id.to_string()))
			}?;
			let mut user = User::new(kind, user_id, json::access_str(user_obj, "name")?);

			match user_obj.get("uuid") {
				Some(uuid) => user.set_uuid(json::ensure_type(uuid.as_str(), JsonType::Str)?),
				None => cprintln!("<y>Warning: It is recommended to have your uuid in the configuration for user {}", user_id)
			};
			
			auth.users.insert(user_id.to_string(), user);
		}
		
		if let Some(user_val) = obj.get("default_user") {
			let user_id = json::ensure_type(user_val.as_str(), JsonType::Str)?.to_string();
			match auth.users.get(&user_id) {
				Some(..) => auth.state = AuthState::Authed(user_id),
				None => return Err(ConfigError::from(ContentError::DefaultUserNotFound(user_id)))
			}
		} else if users.is_empty() {
			cprintln!("<y>Warning: Users are available but no default user is set. Starting in offline mode");
		} else {
			cprintln!("<y>Warning: No users are available. Starting in offline mode");
		}

		// Profiles
		let doc_profiles = json::access_object(obj, "profiles")?;
		for (profile_id, profile_val) in doc_profiles {
			let profile_obj = json::ensure_type(profile_val.as_object(), JsonType::Obj)?;
			let version =  MinecraftVersion::from(json::access_str(profile_obj, "version")?);

			let mut profile = Profile::new(profile_id, &version);
			
			// Instances
			if let Some(instances_val) = profile_obj.get("instances") {
				let doc_instances = json::ensure_type(instances_val.as_object(), JsonType::Obj)?;
				for (instance_id, instance_val) in doc_instances {
					if instances.contains_key(instance_id) {
						return Err(ConfigError::from(ContentError::DuplicateInstance(instance_id.to_string())));
					}

					let instance_obj = json::ensure_type(instance_val.as_object(), JsonType::Obj)?;
					let kind = match json::access_str(instance_obj, "type")? {
						"client" => Ok(InstKind::Client),
						"server" => Ok(InstKind::Server),
						typ => Err(ContentError::InstType(typ.to_string(), instance_id.to_string()))
					}?;

					let instance = Instance::new(kind, instance_id, &version);
					profile.add_instance(instance_id);
					instances.insert(instance_id.to_string(), instance);
				}
			}

			if let Some(packages_val) = profile_obj.get("packages") {
				let doc_packages = json::ensure_type(packages_val.as_array(), JsonType::Arr)?;
				for package_val in doc_packages {
					let package_obj = json::ensure_type(package_val.as_object(), JsonType::Obj)?;
					let package_id = json::access_str(package_obj, "id")?;
					let package_version = match package_obj.get("version") {
						Some(version) => VersionPattern::Single(
							json::ensure_type(version.as_str(), JsonType::Str)?.to_owned()
						),
						None => VersionPattern::Latest(None)
					};
					let req = PkgRequest::new(package_id, &package_version);
					if let Some(val) = package_obj.get("type") {
						match json::ensure_type(val.as_str(), JsonType::Str)? {
							"local" => {
								let package_path = json::access_str(package_obj, "path")?;
								if let VersionPattern::Single(version) = package_version {
									packages.insert_local(
										&PkgIdentifier {name: package_id.to_owned(), version},
										&profile_id,
										&PathBuf::from(package_path)
									);
								} else {
									Err(ContentError::LocalPackageVersion(package_id.to_owned()))?
								}
							},
							"remote" => {}
							typ => Err(ContentError::PkgType(typ.to_string(), String::from("package")))?
						}
					}
					let features = match package_obj.get("features") {
						Some(list) => {
							json::ensure_type(list.as_array(), JsonType::Arr)?;
							let mut out = Vec::new();
							for feature in list.as_array().expect("Features list is not an array") {
								json::ensure_type(feature.as_str(), JsonType::Str)?;
								out.push(feature.as_str().expect("Feature is not a string").to_owned());
							}
							out
						}
						None => Vec::new()
					};
					let pkg = PkgConfig {
						req,
						features
					};
					profile.packages.push(pkg);
				}
			}
			
			profiles.insert(profile_id.to_string(), Box::new(profile));
		}

		Ok(Self {
			auth,
			instances,
			profiles,
			packages,
			prefs
		})
	}

	pub fn load(path: &PathBuf) -> Result<Self, ConfigError> {
		let obj = Self::open(path)?;
		Self::load_from_obj(&obj)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default_config() {
		let obj = json::ensure_type(default_config().as_object(),
			JsonType::Obj).unwrap().clone();
		Config::load_from_obj(&obj).unwrap();
	}
}
