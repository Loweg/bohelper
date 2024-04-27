#![allow(dead_code)]

use std::{
	collections::HashMap,
	io::{BufReader, Read},
	fs::File,
	path::PathBuf,
};

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

#[derive(Clone, Debug)]
pub struct Item {
	pub label: String,
	pub aspects: HashMap<String, isize>,
	pub scrutiny: Option<String>,
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

#[derive(Deserialize, Clone, Debug)]
struct WorkstationFile {
	verbs: Vec<Workstation>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Workstation {
	//pub id: String,
	pub label: String,
	pub slots: Vec<Slot>,
	pub aspects: HashMap<String, isize>,
	pub hints: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Slot {
	pub label: String,
	pub required: HashMap<String, isize>,
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
	aspects: HashMap<String, isize>,
}

#[derive(Clone, Debug)]
pub struct Skill {
	pub label: String,
	pub principles: (String, String),
	pub wisdoms: ((String, String), (String, String)),
}

#[derive(Deserialize, Clone, Debug)]
struct WisdomCommitments {
	recipes: Vec<Commitment>,
}

#[derive(Deserialize, Clone, Debug)]
struct Commitment {
	id: String,
	effects: HashMap<String, isize>,
}

#[derive(Deserialize, Clone, Debug)]
struct RecipeFile {
	recipes: Vec<SerdeRecipe>,
}

#[derive(Deserialize, Clone, Debug)]
struct SerdeRecipe {
	#[serde(rename = "Label")]
	label: String,
	reqs: HashMap<String, isize>,
}

impl SerdeRecipe {
	fn to_recipe(self) -> Recipe {
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
pub struct Recipe {
	pub label:      String,
	pub skill:      String,
	pub principle:  String,
	pub ingredient: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Data {
	pub items:  HashMap<String, Item>,
	pub books:  HashMap<String, Book>,
	pub skills: HashMap<String, Skill>,
	pub workstations: Vec<Workstation>,
	pub recipes: (Vec<Recipe>, Vec<Recipe>, Vec<Recipe>),
}

pub fn principles_from_soul(soul: &String) -> (&'static str, Vec<&'static str>) {
	match soul.as_str() {
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
			aspects,
			scrutiny: item.xtriggers.map(|s| s.scrutiny.get(0).expect("Empty scrutiny").id.clone()),
		});
	}

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
	let recipes_keeper: Vec<_> = recipe_json.recipes.into_iter().map(|r| r.to_recipe()).collect();

	let recipe_rdr = open_data(data_path.clone(), "recipes", "crafting_3_scholar.json");
	let recipe_json: RecipeFile = serde_json::from_reader(recipe_rdr).expect("Failed to parse Scholar recipes");
	let recipes_scholar: Vec<_> = recipe_json.recipes.into_iter().map(|r| r.to_recipe()).collect();

	let recipe_rdr = open_data(data_path.clone(), "recipes", "crafting_4b_prentice.json");
	let recipe_json: RecipeFile = serde_json::from_reader(recipe_rdr).expect("Failed to parse Prentice recipes");
	let recipes_prentice: Vec<_> = recipe_json.recipes.into_iter().map(|r| r.to_recipe()).collect();

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

fn parse_books(book_file: BookFile) -> HashMap<String, Book> {
	let mut books = HashMap::new();
	for book in book_file.elements {
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
