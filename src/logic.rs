use std::collections::{HashMap, HashSet};

use crate::data::*;
use crate::save::WorldItem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Memory {
	pub label: String,
	pub source_label: String,
	pub aspects: AspectMap,
}

pub fn find_memories(
	principles: &[&String],
	world_items: &Vec<WorldItem>,
	items: &HashMap<String, Item>,
	books: &HashMap<String, Book>,
) -> HashMap<String, HashSet<String>> {
	let mut mems = HashMap::new();

	for item in world_items {
		// Scrutiny
		if let Some(item) = items.get(&item.id) {
			if let Some(mem) = item.scrutiny.clone().and_then(|s| items.get(&s)) {
				if mem.aspects.keys().any(|a| principles.contains(&a)) {
					ins_ext(&mut mems, &mem.label, &item.label);
				}
			}
			// Beast
			if let ExhaustType::Beast(b) = &item.fatigues {
				let mem = items.get(b).expect("Couldn't find item for item ID");
				if mem.aspects.keys().any(|a| principles.contains(&a)) {
					ins_ext(&mut mems, &mem.label, &item.label);
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
						ins_ext(&mut mems, &mem.label, &book.label);
					}
				}
				break;
			}
		}
	}
	mems
}

fn ins_ext(map: &mut HashMap<String, HashSet<String>>, mem: &String, item: &String) {
	if let Err(mut e) = map.try_insert(mem.clone(), HashSet::from([item.clone()])) {
		e.entry.get_mut().insert(item.clone());
	}
}

pub fn get_skill_stations(skills: &Vec<&Skill>, workstations: &[Workstation]) -> String {
	let mut res = String::new();
	for skill in skills {
		res.push_str(&format!("<h3>{}</h3>", skill.label));
		res.push_str(&add_commitment(&skill.wisdoms.0, skill, workstations));
		res.push_str(&add_commitment(&skill.wisdoms.1, skill, workstations));
	}
	res
}

fn add_commitment(commit: &(String, String), skill: &Skill, workstations: &[Workstation]) -> String {
	let wisdom = commit.0.split('.').nth(1)	.unwrap();
	let soul = principles_from_soul(&commit.1);
	let id = "e.".to_string() + wisdom;
	let stations: Vec<_> = workstations.iter().filter(|w|
		w.wisdoms.contains(&id) &&
		w.accepts_principles(&[&skill.principles.0, &skill.principles.1]) &&
		w.accepts_principles(soul.1.as_slice()))
		.map(|w| w.label.clone()).collect();
	match stations.len() {
		0 => format!("<p>Warning: {} can't be upgraded when committed to {}</p>", soul.0, wisdom),
		_ => format!(         "<p>{} is upgraded at {} when committed to {}</p>", soul.0, dis_vec(&stations), wisdom),
	}
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

pub fn dis_vec(v: &Vec<String>) -> String {
	match v.len() {
		0 => String::new(),
		1 => v[0].to_string(),
		2 => format!("{} or {}", v[0].clone(), v[1].clone()),
		l => {
			let mut res = String::new();
			for i in 0..(l-1) {
				res.push_str(&fmt_name(v[i].to_string()));
				res.push_str(", ");
			}
			res.push_str("or ");
			res.push_str(&fmt_name(v[l-1].to_string()));
			res
		}
	}
}

pub fn dis_set(v: &HashSet<String>) -> String {
	match v.len() {
		0 => String::new(),
		1 => v.iter().next().unwrap().to_string(),
		2 => {
			let mut it = v.iter();
			format!("{} or {}", it.next().unwrap().to_string(), it.next().unwrap().to_string())
		},
		_ => {
			let mut qty = 0;
			let mut res = String::new();
			let mut it = v.iter().peekable();
			loop {
				let item = it.next().unwrap();
				if it.peek().is_none() || qty == 8 {
					res.push_str("or ");
					res.push_str(&fmt_name(item.to_string()));
					return res;
				}
				res.push_str(&fmt_name(item.to_string()));
				res.push_str(", ");
				qty += 1;
			}
		}
	}
}

fn fmt_name(name: String) -> String {
	if name.contains(',') {
		format!("'{}'", name)
	} else { name }
}
