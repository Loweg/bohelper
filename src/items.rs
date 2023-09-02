use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
struct PrototypeFile {
	elements: Vec<SerdePrototype>
}

#[derive(Deserialize, Clone, Debug)]
struct SerdePrototype {
	id: String,
	aspects: Option<HashMap<String, isize>>,
}

#[derive(Deserialize, Clone, Debug)]
struct ItemFile {
	elements: Vec<SerdeItem>
}

#[derive(Deserialize, Clone, Debug)]
struct SerdeItem {
	#[serde(rename = "ID")]
	id: String,
	#[serde(rename = "Label")]
	label: String,
	aspects: HashMap<String, isize>,
	unique: Option<bool>,
	inherits: String,
}

pub struct Item {
	pub label: String,
	pub aspects: HashMap<String, isize>,
	pub unique: bool,
}

#[derive(Deserialize, Clone, Debug)]
struct BookFile {
	elements: Vec<SerdeBook>
}

#[derive(Deserialize, Clone, Debug)]
struct SerdeBook {
	#[serde(rename = "ID")]
	id: Option<String>,
	#[serde(rename = "Label")]
	label: Option<String>,
	aspects: Option<HashMap<String, isize>>,
	xtriggers: Option<HashMap<String, Vec<Trigger>>>
}

#[derive(Deserialize, Clone, Debug)]
struct Trigger {
	id: String,
	level: isize,
}

#[derive(Clone, Debug)]
pub struct Book {
	pub label: String,
	pub aspects: HashMap<String, isize>,
	pub skill: (String, isize),
	pub memory: String,
}

pub fn init_items(data_path: &PathBuf) -> (HashMap<String, Item>, HashMap<String, Book>) {
	let prototypes_rdr = open_data(data_path.clone(), "elements\\_prototypes.json");
	let prototypes_json: PrototypeFile = serde_json::from_reader(prototypes_rdr).expect("Failed to parse prototypes file");

	let mut prototypes = HashMap::new();
	for prototype in prototypes_json.elements {
		if let Some(ext) = prototype.aspects {
			prototypes.insert(prototype.id, ext);
		}
	}

	let items_rdr = open_data(data_path.clone(), "elements\\aspecteditems.json");
	let items_json: ItemFile = serde_json::from_reader(items_rdr).expect("Failed to parse items file");

	let mut items = HashMap::new();

	for item in items_json.elements {
		let mut aspects = item.aspects;
		if let Some(ext) = prototypes.get(&item.inherits) {
			for (aspect, intensity) in ext {
				aspects.insert(aspect.clone(), *intensity);
			}
		}
		items.insert(item.id, Item {
			label: item.label,
			aspects: aspects,
			unique: item.unique.unwrap_or(false),
		});
	}

	let books_rdr = open_data(data_path.clone(), "elements\\tomes.json");
	let books_json: BookFile = serde_json::from_reader(books_rdr).expect("Failed to parse tomes file");

	let mut books = HashMap::new();
	for book in books_json.elements {
		if let None = book.id {
			continue
		};

		let mut skill = None;
		let mut memory = None;
		for (trigger, res) in book.xtriggers.unwrap().drain() {
			if trigger.starts_with("mastering") {
				if res.len() != 1 {
					println!("Warning: Tome: mastering len was {}. Tome ID: {}", res.len(), book.id.clone().unwrap())
				}
				skill = Some((res[0].id.clone(), res[0].level));
			} else if trigger.starts_with("reading") {
				if res.len() != 1 {
					println!("Warning: Tome: reading len was {}. Tome ID: {}", res.len(), book.id.clone().unwrap())
				}
				memory = Some(res[0].id.clone())
			}
		}

		if skill == None {
			panic!("No skill returned for book {}", book.label.unwrap())
		}
		if memory == None {
			panic!("No memory returned for book {}", book.label.unwrap())
		}

		books.insert(book.id.unwrap(), Book {
			label: book.label.unwrap(),
			aspects: book.aspects.unwrap(),
			skill: skill.unwrap(),
			memory: memory.unwrap(),
		});
	}

	(items, books)
}

fn open_data(path: PathBuf, location: &str) -> BufReader<File> {
	let mut p = path.clone();
	p.push(location);
	let file = match File::open(&p) {
		Ok(f) => f,
		Err(_) => panic!("Failed to open game data at {}", p.to_string_lossy())
	};
	BufReader::new(file)
}