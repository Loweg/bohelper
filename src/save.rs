use std::{
	collections::HashMap,
	fs::File,
	io::BufReader,
	path::PathBuf
};

use serde::Deserialize;
use serde_json::Value;

#[allow(dead_code)]
pub struct SaveData {
	pub items: Vec<WorldItem>,
	pub skills: Vec<String>,
	pub abilities: Vec<String>,
}

impl SaveData {
	pub fn from_path(path: PathBuf) -> Self {
		let save_file = File::open(path).expect("Failed to open save file");
		let save_rdr = BufReader::new(save_file);
		let save: Save = serde_json::from_reader(save_rdr).expect("Failed to parse save file");
		save.resolve()
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Save {
	root_population_command: Dominion,
	populate_xamanek_command: World,
}

impl Save {
	fn resolve(self) -> SaveData {
		let mut environs = self.populate_xamanek_command.current_enviro_fx_commands;
		let non_locations = ["$type", "vignette", "sky", "weather", "music", "ui_watcher_buttons", "season", "ui_wisdoms_or_world", "meta", "run"];
		for non in non_locations {
			environs.remove(non);
		}
		let locations: Vec<_> = environs.into_iter().map(|e| e.0).collect();

		let item_spheres = ["fixedverbs", "portage1", "portage2", "portage3", "portage4", "portage5", "TerrainDetailInputSphere", "hand.misc", "hand.memories"];

		let mut world_items = Vec::new();
		let mut skills = Vec::new();
		let mut abilities = Vec::new();
		for sphere in self.root_population_command.spheres {
			let id = sphere.governing_sphere_spec.id.clone();
			if item_spheres.contains(&id.as_str()) {
				for payload in sphere.resolve() {
					world_items.push(WorldItem {
						id: payload.entity_id.expect("No entity ID"),
						mutations: payload.mutations,
					})
				}
			} else if id == "Library" {
				for token in &sphere.tokens {
					if locations.contains(&token.payload.id) {
						if token.payload.id == "brancrug" {
							for dominion in &token.payload.dominions {
								for sphere in &dominion.spheres {
									if !sphere.governing_sphere_spec.id.starts_with("ChristmasSlot") {
										for payload in sphere.resolve() {
											world_items.push(WorldItem {
												id: payload.entity_id.expect("No entity ID"),
												mutations: payload.mutations,
											})
										}
									}
								}
							}
							continue;
						}
						for payload in token.resolve() {
							world_items.push(WorldItem {
								id: payload.entity_id.expect("No entity ID"),
								mutations: payload.mutations,
							})
						}
					}
				}
			} else if id == "hand.skills" {
				for payload in sphere.resolve() {
					skills.push(payload.entity_id.unwrap());
				}
			} else if id == "hand.abilities" {
				for payload in sphere.resolve() {
					abilities.push(payload.entity_id.unwrap());
				}
			}
		}
		SaveData {
			items: world_items,
			skills,
			abilities,
		}
	}
}

pub struct WorldItem {
	pub id: String,
	pub mutations: HashMap<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct World {
	current_enviro_fx_commands: HashMap<String, Value>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct Dominion {
	spheres: Vec<Sphere>,
}

impl Dominion {
	fn resolve(&self) -> Vec<Payload> {
		let mut payloads = Vec::new();
		for sphere in &self.spheres {
			payloads.extend(sphere.resolve())
		}
		payloads
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct Sphere {
	governing_sphere_spec: Spec,
	tokens: Vec<Token>
}

impl Sphere {
	fn resolve(&self) -> Vec<Payload> {
		let mut payloads = Vec::new();
		for token in &self.tokens {
			payloads.extend(token.resolve())
		}
		payloads
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct Spec {
	id: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct Token {
	payload: Payload,
}

impl Token {
	fn resolve(&self) -> Vec<Payload> {
		match &self.payload.dominions.len() {
			0 => vec![self.payload.clone()],
			_ => {
				let mut payloads = Vec::new();
				for dominion in &self.payload.dominions {
					payloads.extend(dominion.resolve())
				}
				payloads
			}
		}
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct Payload {
	id: String,
	entity_id: Option<String>,
	dominions: Vec<Dominion>,
	mutations: HashMap<String, Value>,
}

#[cfg(windows)]
pub fn default_save_path() -> PathBuf {
	use windows::Storage::UserDataPaths;
	let mut path = UserDataPaths::GetDefault().expect("Failed to acquire UserDataPaths instance")
		.LocalAppDataLow().expect("Failed to find LocalLow path").to_os_string();
	path.push("Weather Factory");
	path.push("Book of Hours");
	path.push("AUTOSAVE.json");
	PathBuf::from(path)
}

#[cfg(unix)]
pub fn default_save_path() -> PathBuf {
	PathBuf::from("/Users/loweg/Library/Application Support/Weather Factory/Book of Hours/AUTOSAVE.json")
}
