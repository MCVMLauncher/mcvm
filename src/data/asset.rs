#[derive(Debug, Clone)]
pub enum AssetKind {
	ResourcePack,
	Datapack,
	Mod,
	Plugin,
	World
}

impl AssetKind {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"resource_pack" => Some(Self::ResourcePack),
			"datapack" => Some(Self::Datapack),
			"mod" => Some(Self::Mod),
			"plugin" => Some(Self::Plugin),
			"world" => Some(Self::World),
			_ => None
		}
	}
}

pub struct Asset {
	pub kind: AssetKind,
	pub name: String
}

impl Asset {
	pub fn new(kind: AssetKind, name: &str) -> Self {
		Self {
			kind,
			name: name.to_owned()
		}
	}
}

#[derive(Debug, Clone)]
pub enum Modloader {
	Forge,
	Fabric
}

impl Modloader {
	pub fn from_str(string: &str) -> Option<Self> {
		match string {
			"forge" => Some(Self::Forge),
			"fabric" => Some(Self::Fabric),
			_ => None
		}
	}
}