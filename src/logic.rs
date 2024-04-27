use std::collections::{HashMap, HashSet};

use crate::{data::{Book, Item}, save::WorldItem};

pub struct Memory {
	pub label: String,
	pub source_label: String,
	pub aspects: HashMap<String, isize>,
}

pub fn find_memories(
	principle: &String,
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

		// Scrutiny
		if let Some(data) = items.get(&item.id) {
			if let Some(s) = &data.scrutiny {
				let mem = items.get(s).expect("Memory not found");
				if mem.aspects.contains_key(principle) {
					idx.insert(s.clone());
					qty_remaining -= 1;
					mems.push(Memory{
						label: mem.label.clone(),
						source_label: data.label.clone(),
						aspects: mem.aspects.clone(),
					});
					continue;
				}
			}
		}

		// Book
		for (k, _) in &item.mutations {
			if k.starts_with("mastery") {
				if let Some(book) = books.get(&item.id.clone()) {
					let mem = items.get(&book.memory).expect("Couldn't find item for item ID");
					if mem.aspects.contains_key(principle) {
						idx.insert(book.memory.clone());
						qty_remaining -= 1;
						mems.push(Memory{
							label: mem.label.clone(),
							source_label: book.label.clone(),
							aspects: mem.aspects.clone(),
						});
					}
				}
				break;
			}
		}
	}
	mems
}
