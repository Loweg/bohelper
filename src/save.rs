use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Save {
	pub root_population_command: Dominion,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Dominion {
	spheres: Vec<Sphere>,
}

impl Dominion {
	pub fn resolve(&self) -> Vec<Payload> {
		let mut payloads = Vec::new();
		for sphere in &self.spheres {
			payloads.extend(sphere.resolve())
		}
		payloads
	}
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Sphere {
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
pub struct Token {
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
pub struct Payload {
	pub entity_id: Option<String>,
	pub dominions: Vec<Dominion>,
	pub mutations: HashMap<String, Value>,
	pub is_shrouded: Option<bool>,
}
