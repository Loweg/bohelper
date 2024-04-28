//#![allow(dead_code)]

use std::collections::HashMap;

use ::serde::Deserialize;

mod read;
pub use read::{init_items, Data};

pub type AspectMap = HashMap<String, isize>;

#[derive(Clone, Debug)]
pub enum ExhaustType {
	Beast(String),
	No,
	Yes,
}

impl ExhaustType {
	pub fn exhausts(&self) -> bool {
		match self {
			ExhaustType::Beast(_) | ExhaustType::No => false,
			ExhaustType::Yes => false,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Item {
	pub label: String,
	pub aspects: AspectMap,
	pub scrutiny: Option<String>,
	pub fatigues: ExhaustType,
}

#[derive(Clone, Debug)]
pub struct Book {
	pub label: String,
	pub aspects: AspectMap,
	pub skill: (String, isize),
	pub memory: String,
}

#[derive(Clone, Debug)]
pub struct Skill {
	pub label: String,
	pub principles: (String, String),
	pub wisdoms: ((String, String), (String, String)),
}

impl Skill {
	pub fn matches(&self, aspects: &[String]) -> bool {
		aspects.contains(&self.principles.0) || aspects.contains(&self.principles.1)
	}

	pub fn matches_exact(&self, aspects: &[String]) -> bool {
		aspects.contains(&self.principles.0) && aspects.contains(&self.principles.1)
	}
}

#[derive(Deserialize, Clone, Debug)]
pub struct Workstation {
	pub label: String,
	pub slots: Vec<Slot>,
	pub aspects: AspectMap,
	pub hints: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Slot {
	pub label: String,
	pub required: AspectMap,
}

#[derive(Clone, Debug)]
pub struct Recipe {
	pub label:      String,
	pub skill:      String,
	pub principle:  String,
	pub ingredient: Option<String>,
}

pub fn principles_from_soul(soul: &str) -> (&'static str, Vec<&'static str>) {
	match soul {
		"xcho" => ("Chor", vec!["heart", "grail"]),
		"xere" => ("Ereb", vec!["grail", "edge"]),
		"xfet" => ("Fet", vec!["rose", "moth"]),
		"xhea" => ("Health", vec!["heart", "nectar", "scale"]),
		"xmet" => ("Mettle", vec!["forge", "edge"]),
		"xpho" => ("Phost", vec!["lantern", "sky"]),
		"xsha" => ("Shapt", vec!["knock", "forge"]),
		"xtri" => ("Trist", vec!["moth", "moon"]),
		"xwis" => ("Wist", vec!["winter", "lantern"]),
		s      => panic!("Unexpected Element of the Soul: {}", s),
	}
}

pub fn principles() -> Vec<&'static str> {
	vec!["edge", "forge", "grail", "heart", "knock", "lantern", "moon", "moth", "nectar", "rose", "scale", "sky", "winter"]
}
