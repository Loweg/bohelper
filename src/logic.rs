use std::collections::HashMap;

use crate::{
	data::{
		principles_from_soul,
		AspectMap,
		Book,
		ExhaustType,
		Item,
		Skill,
		Workstation
	},
	save::WorldItem
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Memory {
	pub label: String,
	pub source_label: String,
	pub aspects: AspectMap,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord)]
enum Source {
	NonExhaust,
	Book,
	Beast,
	Exhaust,
}
impl PartialOrd for Source {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some((*self as u8).cmp(&(*other as u8)))
	}
}

pub fn find_memories(
	principles: &[&String],
	qty: u8,
	world_items: &Vec<WorldItem>,
	items: &HashMap<String, Item>,
	books: &HashMap<String, Book>,
) -> HashMap<String, (String, AspectMap)> {
	let mut candidates = Vec::new();

	for item in world_items {
		// Scrutiny
		if let Some(item) = items.get(&item.id) {
			if let Some(mem) = item.scrutiny.clone().and_then(|s| items.get(&s)) {
				if mem.aspects.keys().any(|a| principles.contains(&a)) {
					let source = if item.fatigues.exhausts() { Source::Exhaust } else { Source::NonExhaust };
					candidates.push((Memory {
						label: mem.label.clone(),
						source_label: item.label.clone(),
						aspects: mem.aspects.clone(),
					}, source));
				}
			}
			// Beast
			if let ExhaustType::Beast(b) = &item.fatigues {
				let mem = items.get(b).expect("Couldn't find item for item ID");
				if mem.aspects.keys().any(|a| principles.contains(&a)) {
					candidates.push((Memory {
						label: mem.label.clone(),
						source_label: item.label.clone(),
						aspects: mem.aspects.clone(),
					}, Source::Beast));
				}
			}
			continue;
		}

		// Book
		for k in item.mutations.keys() {
			if k.starts_with("mastery") {
				if let Some(book) = books.get(&item.id) {
					let mem = items.get(&book.memory).expect("Couldn't find item for item ID");
					if mem.aspects.keys().any(|a| principles.contains(&a)) {
						candidates.push((Memory {
							label: mem.label.clone(),
							source_label: book.label.clone(),
							aspects: mem.aspects.clone(),
						}, Source::Book));
					}
				}
				break;
			}
		}
	}

	candidates.sort_by_key(|x| x.1);
	candidates.into_iter().map(|(m, _)| (m.label, (m.source_label, m.aspects))).take(qty.into()).collect()
}

pub fn get_skill_stations(skills: &Vec<&Skill>, workstations: &[Workstation]) -> String {
	let mut res = String::new();
	for skill in skills {
		res.push_str(&skill.label);
		res.push('\n');
		add_commitment(&skill.wisdoms.0, skill, workstations, &mut res);
		add_commitment(&skill.wisdoms.1, skill, workstations, &mut res);
		res.push('\n')
	}
	res
}

fn add_commitment(commit: &(String, String), skill: &Skill, workstations: &[Workstation], res: &mut String) {
	let wisdom = commit.0.split('.').nth(1)	.unwrap();
	let soul = principles_from_soul(&commit.1);
	let id = "e.".to_string() + wisdom;
	let stations: Vec<_> = workstations.iter().filter(|w|
		w.wisdoms.contains(&id) &&
		w.accepts_principles(&[&skill.principles.0, &skill.principles.1]) &&
		w.accepts_principles(soul.1.as_slice()))
		.map(|w| &w.label).collect();
	let s = match stations.len() {
		1 => format!(                 "{} is upgraded at {} when committed to {}\n", soul.0, stations[0], wisdom),
		2 => format!(           "{} is upgraded at {} or {} when committed to {}\n", soul.0, stations[0], stations[1], wisdom),
		0 => format!("Warning: {} can't be upgraded with {} when committed to {}\n", soul.0, skill.label, wisdom),
		_ => format!(               "{} is upgraded at {:?} when committed to {}\n", soul.0, stations, wisdom),
		};
		res.push_str(&s);
}

pub fn find_aspected(items: &HashMap<String, Item>, aspects: &[&str]) -> HashMap<String, AspectMap> {
	let mut found = HashMap::new();

	for item in items.values() {
		let label = if item.label.starts_with("Lepidoptery") || item.label.starts_with("Wire") {
			&item.label
		} else {
			item.label.split('(').next().unwrap()
		};
		if found.contains_key(label) { continue; }

		if aspects.iter().all(|a| item.aspects.contains_key(a.to_owned())) {
			found.insert(label.to_owned(), item.aspects.clone());
		}
	}
	found
}
