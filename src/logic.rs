use std::collections::{HashMap, HashSet};

use crate::{data::{Book, Item}, save::WorldItem};

pub struct Memory {
	pub label: String,
	pub source_label: String,
	pub aspects: HashMap<String, isize>,
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
		if let Some(item) = items.get(&item.id) {
			if let Some(s) = &item.scrutiny {
				let mem = items.get(s).expect("Memory not found");
				if insert_mem(mem, s.clone(), item.label.clone()) {
					continue;
				}
			}
		}

		// Book
		for (k, _) in &item.mutations {
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
