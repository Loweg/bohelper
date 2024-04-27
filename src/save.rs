use std::{
	collections::HashMap,
	path::PathBuf,
};

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Save {
	root_population_command: Dominion,
}

impl Save {
	pub fn resolve(&self) -> Vec<WorldItem> {
		let mut world_items = Vec::new();
		for payload in self.root_population_command.resolve() {
			world_items.push(WorldItem {
				id: payload.entity_id.expect("No entity ID"),
				mutations: payload.mutations,
			})
		}
		world_items
	}
}

pub struct WorldItem {
	pub id: String,
	pub mutations: HashMap<String, Value>,
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
