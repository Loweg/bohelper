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
		aspects.contains(&self.principles.0) && aspects.contains(&self.principles.1)
	}
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SpecialWorkstation {
	Kitchen,
	Instrument,
	None,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Workstation {
	pub label: String,
	pub principles: Vec<String>,
	pub subject: Vec<String>,
	pub with: Vec<String>,
	pub wisdoms: Vec<String>,
	pub special: SpecialWorkstation,
}

impl Workstation {
	pub fn can_craft(&self, recipe: &Recipe, data: &Data) -> bool {
		let principles = &data.skills.get(&recipe.skill).unwrap().principles;
		(if let Some(ingredient) = &recipe.ingredient {
			if let Some(item) = data.items.get(ingredient) {
				self.accepts_item(item)
			} else {
				self.accepts_aspect(ingredient)
			}
		} else { true }) && self.accepts_principles(&[&principles.0, &principles.1])
	}

	pub fn accepts_principles(&self, principles: &[&str]) -> bool {
		self.principles.iter().any(|p| principles.contains(&p.as_str()))
	}

	fn accepts_aspect(&self, aspect: &String) -> bool {
		match aspect.as_str() {
			"instrument" => self.special == SpecialWorkstation::Instrument,
			"kitchenware" | "knife" | "egg" => self.special == SpecialWorkstation::Kitchen,
			_ => self.subject.iter().chain(self.with.iter()).any(|a| a == aspect)
		}
	}

	fn accepts_item(&self, item: &Item) -> bool {
		item.aspects.keys().any(|a| self.accepts_aspect(a))
	}
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
