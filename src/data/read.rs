use std::{
	collections::HashMap,
	io::{BufReader, Read},
	fs::File,
	path::PathBuf,
};

use serde::Deserialize;

use super::*;

#[derive(Deserialize, Clone, Debug)]
struct PrototypeFile {
	elements: Vec<SerdePrototype>
}

#[derive(Deserialize, Clone, Debug)]
struct SerdePrototype {
	id: String,
	aspects: Option<AspectMap>,
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
	aspects: AspectMap,
	xtriggers: Option<XTrigger>,
	inherits: String,
}

#[derive(Deserialize, Clone, Debug)]
struct XTrigger {
	scrutiny: Vec<Scrutiny>,
}

#[derive(Deserialize, Clone, Debug)]
struct Scrutiny {
	id: String,
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
	aspects: Option<AspectMap>,
	xtriggers: Option<HashMap<String, Vec<Trigger>>>
}

#[derive(Deserialize, Clone, Debug)]
struct Trigger {
	id: String,
	level: isize,
}

#[derive(Deserialize, Clone, Debug)]
struct WorkstationFile {
	verbs: Vec<Workstation>,
}

#[derive(Deserialize, Clone, Debug)]
struct SkillFile {
	elements: Vec<SerdeSkill>,
}

#[derive(Deserialize, Clone, Debug)]
struct SerdeSkill {
	id: String,
	#[serde(rename = "Label")]
	label: String,
	aspects: AspectMap,
}

#[derive(Deserialize, Clone, Debug)]
struct WisdomCommitments {
	recipes: Vec<Commitment>,
}

#[derive(Deserialize, Clone, Debug)]
struct Commitment {
	id: String,
	effects: AspectMap,
}

#[derive(Deserialize, Clone, Debug)]
struct RecipeFile {
	recipes: Vec<SerdeRecipe>,
}

#[derive(Deserialize, Clone, Debug)]
struct SerdeRecipe {
	#[serde(rename = "Label")]
	label: String,
	reqs: AspectMap,
}

impl SerdeRecipe {
	fn into_recipe(self) -> Recipe {
		let mut skill = None;
		let mut principle = None;
		let mut ingredient = None;
		for (k, _) in self.reqs {
			if k == "ability" { continue }
			else if k.starts_with("s.") {
				skill = Some(k);
			} else if principles().contains(&k.as_str()) {
				principle = Some(k);
			} else {
				ingredient = Some(k);
			}
		}
		Recipe {
			label: self.label,
			skill: skill.expect("Recipe: No skill found"),
			principle: principle.expect("Recipe: No principle found"),
			ingredient,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Data {
	pub items:  HashMap<String, Item>,
	pub books:  HashMap<String, Book>,
	pub skills: HashMap<String, Skill>,
	pub workstations: Vec<Workstation>,
	pub recipes: (Vec<Recipe>, Vec<Recipe>, Vec<Recipe>),
}

pub fn init_items(data_path: &PathBuf) -> Data {
	let prototypes_rdr = open_data(data_path.clone(), "elements", "_prototypes.json");
	let prototypes_json: PrototypeFile = serde_json::from_reader(prototypes_rdr).expect("Failed to parse prototypes file");

	let mut prototypes = HashMap::new();
	for prototype in prototypes_json.elements {
		if let Some(ext) = prototype.aspects {
			prototypes.insert(prototype.id, ext);
		}
	}

	let mut item_path = data_path.clone();
	item_path.push("elements");
	item_path.push("aspecteditems.json");
	let mut f = File::open(item_path).unwrap();
	let mut data = Vec::new();
	f.read_to_end(&mut data).unwrap();

	let items_data = String::from_utf16le(&data[2..]).unwrap();
	let items_json: ItemFile = serde_json::from_str(&items_data).expect("Failed to parse items file");
	let items = parse_items(items_json, prototypes);

	let mut book_path = data_path.clone();
	book_path.push("elements");
	book_path.push("tomes.json");
	let mut bf = File::open(book_path).unwrap();
	let mut b_data = Vec::new();
	bf.read_to_end(&mut b_data).unwrap();

	let books_data = String::from_utf16le(&b_data[2..]).unwrap();
	let books_json: BookFile = serde_json::from_str(&books_data).expect("Failed to parse tomes file");
	let books = parse_books(books_json);

	let workstations_rdr = open_data(data_path.clone(), "verbs", "workstations_library_world.json");
	let workstations_json: WorkstationFile = serde_json::from_reader(workstations_rdr).expect("Failed to parse workstations file");

	let wis_rdr = open_data(data_path.clone(), "recipes", "wisdom_commitments.json");
	let wis_json: WisdomCommitments = serde_json::from_reader(wis_rdr).expect("Failed to parse wisdom commitments file");
	let commitments: HashMap<_, _> = wis_json.recipes.into_iter().map(|w| (w.id.clone(), w)).collect();

	let skills_rdr = open_data(data_path.clone(), "elements", "skills.json");
	let skills_json: SkillFile = serde_json::from_reader(skills_rdr).expect("Failed to parse skills file");
	let skills = parse_skills(skills_json, commitments);

	let recipe_rdr = open_data(data_path.clone(), "recipes", "crafting_2_keeper.json");
	let recipe_json: RecipeFile = serde_json::from_reader(recipe_rdr).expect("Failed to parse Keeper recipes");
	let recipes_keeper: Vec<_> = recipe_json.recipes.into_iter().map(|r| r.into_recipe()).collect();

	let recipe_rdr = open_data(data_path.clone(), "recipes", "crafting_3_scholar.json");
	let recipe_json: RecipeFile = serde_json::from_reader(recipe_rdr).expect("Failed to parse Scholar recipes");
	let recipes_scholar: Vec<_> = recipe_json.recipes.into_iter().map(|r| r.into_recipe()).collect();

	let recipe_rdr = open_data(data_path.clone(), "recipes", "crafting_4b_prentice.json");
	let recipe_json: RecipeFile = serde_json::from_reader(recipe_rdr).expect("Failed to parse Prentice recipes");
	let recipes_prentice: Vec<_> = recipe_json.recipes.into_iter().map(|r| r.into_recipe()).collect();

	Data {
		items,
		books,
		skills,
		workstations: workstations_json.verbs,
		recipes: (recipes_prentice, recipes_scholar, recipes_keeper),
	}
}

fn open_data(path: PathBuf, dir: &str, file: &str) -> BufReader<File> {
	let mut p = path.clone();
	p.push(dir);
	p.push(file);
	let file = match File::open(&p) {
		Ok(f) => f,
		Err(_) => panic!("Failed to open game data at {}", p.to_string_lossy())
	};
	BufReader::new(file)
}

fn parse_items(item_file: ItemFile, prototypes: HashMap<String, AspectMap>) -> HashMap<String, Item> {
	let mut items = HashMap::new();

	for item in item_file.elements {
		let mut aspects = item.aspects;
		if let Some(ext) = prototypes.get(&item.inherits) {
			for (aspect, intensity) in ext {
				aspects.insert(aspect.clone(), *intensity);
			}
		};
		let scrutiny = item.xtriggers.filter(|t| t.scrutiny.first().is_some_and(|s| s.id.is_empty())).map(|t| t.scrutiny.first().unwrap().id.clone());
		items.insert(item.id, Item {
			label: item.label,
			aspects,
			scrutiny,
		});
	}
	items
}

fn parse_books(book_file: BookFile) -> HashMap<String, Book> {
	let mut books = HashMap::new();
	for book in book_file.elements {
		if book.id.is_none() {
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

		if skill.is_none() {
			panic!("No skill returned for book {}", book.label.unwrap())
		}
		if memory.is_none() {
			panic!("No memory returned for book {}", book.label.unwrap())
		}

		books.insert(book.id.unwrap(), Book {
			label: book.label.unwrap(),
			aspects: book.aspects.unwrap(),
			skill: skill.unwrap(),
			memory: memory.unwrap(),
		});
	}
	books
}

fn parse_skills(skill_file: SkillFile, commitments: HashMap<String, Commitment>) -> HashMap<String, Skill> {
	let mut skills = HashMap::new();
	for skill in skill_file.elements {
		let mut p = skill.aspects.clone().into_iter().filter(|a| principles().contains(&a.0.as_str()));
		let w = skill.aspects.into_iter().filter(|a| a.0.starts_with("w."));
		let mut commits = Vec::new();
		for wisdom in w {
			let wis = match wisdom.0.as_str() {
				"w.birdsong"      => "bir",
				"w.bosk"          => "bos",
				"w.horomachistry" => "hor",
				"w.hushery"       => "hus",
				"w.illumination"  => "ill",
				"w.ithastry"      => "ith",
				"w.nyctodromy"    => "nyc",
				"w.preservation"  => "pre",
				"w.skolekosophy"  => "sko",
				w                 => panic!("Unexpected wisdom: {}", w),
			};
			let id = format!("commit.{}.{}", wis, skill.id);
			let effect = commitments.get(&id).expect("Couldn't find wisdom").effects.iter().next().expect("Wisdom commitment has no effects").0.clone();
			commits.push((wisdom.0.clone(), effect));
		}
		let mut c = commits.into_iter();
		skills.insert(skill.id, Skill {
			label: skill.label,
			principles: (p.next().unwrap().0, p.next().unwrap().0),
			wisdoms: (c.next().unwrap(), c.next().unwrap())
		});
	}
	skills
}
