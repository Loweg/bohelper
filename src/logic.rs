use std::collections::{HashMap, HashSet};

use crate::{
	data::{principles_from_soul, AspectMap, Book, Item, Skill, Workstation},
	save::WorldItem
};

pub struct Memory {
	pub label: String,
	pub source_label: String,
	pub aspects: AspectMap,
}

pub fn find_memories(
	principles: &[&String],
	qty: u8,
	world_items: &Vec<WorldItem>,
	items: &HashMap<String, Item>,
	books: &HashMap<String, Book>,
) -> Vec<Memory> {
	let mut mems = Vec::new();
	let mut idx = HashSet::new();
	let mut qty_remaining = qty;

	for item in world_items {
		if qty_remaining == 0 {
			return mems;
		}

		let mut insert_mem = |mem: &Item, id: String, source: String| -> bool {
			if !idx.contains(&id) && mem.aspects.iter().any(|(a, _)| principles.contains(&a)) {
				idx.insert(id);
				qty_remaining -= 1;
				mems.push(Memory{
					label: mem.label.clone(),
					source_label: source,
					aspects: mem.aspects.clone(),
				});
				true
			} else { false }
		};

		// Scrutiny
		/*if let Some(item) = items.get(&item.id) {
			if let Some(s) = &item.scrutiny {
				if let Some(mem) = items.get(s) {
					if insert_mem(mem, s.clone(), item.label.clone()) {
						continue;
					}
				}
			}
		}*/

		// Book
		for k in item.mutations.keys() {
			if k.starts_with("mastery") {
				if let Some(book) = books.get(&item.id.clone()) {
					let mem = items.get(&book.memory).expect("Couldn't find item for item ID");
					if insert_mem(mem, book.memory.clone(), book.label.clone()) {
						continue;
					}
				}
				break;
			}
		}
	}
	mems
}

pub fn print_skill_stations(skills: &Vec<&Skill>, workstations: &[Workstation]) {
	for skill in skills {
		println!("{}", skill.label);
		print_commitment(&skill.wisdoms.0, skill, workstations);
		print_commitment(&skill.wisdoms.1, skill, workstations);
		println!();
	}
}

fn print_commitment(commit: &(String, String), skill: &Skill, workstations: &[Workstation]) {
	let wisdom = commit.0.split('.').nth(1)	.unwrap();
	let soul = principles_from_soul(&commit.1);
	let id = "e.".to_string() + wisdom;
	let stations: Vec<_> = workstations.iter().filter(|w| w.aspects.contains_key(&id))
		.filter(|w| skill.matches(&w.hints) && soul.1.iter().any(|a| w.hints.iter().any(|b| b == a)))
		.map(|w| &w.label).collect();
	match stations.len() {
		1 => println!(                 "{} is upgraded at {} when committed to {}", soul.0, stations[0], wisdom),
		0 => println!("Warning: {} can't be upgraded with {} when committed to {}", soul.0, skill.label, wisdom),
		_ => println!(               "{} is upgraded at {:?} when committed to {}", soul.0, stations, wisdom),
	}
}

pub fn print_aspected(items: &HashMap<String, Item>, aspects: &[String]) {
	let mut found = HashMap::new();

	for item in items.values() {
		let label = if item.label.starts_with("Lepidoptery") || item.label.starts_with("Wire") {
			&item.label
		} else {
			item.label.split('(').next().unwrap()
		};
		if found.contains_key(label) { continue; }

		if aspects.iter().all(|a| item.aspects.contains_key(a)) {
			println!();
			println!("{}", &label);
			found.insert(label, &item.aspects);
			for (aspect, intensity) in &item.aspects {
				if !aspect.starts_with("boost") {
					print!("{aspect}: {intensity}  	");
				}
			}
		}
	}
}
