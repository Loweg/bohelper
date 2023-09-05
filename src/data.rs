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

#[derive(Clone, Debug)]
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

#[derive(Deserialize, Clone, Debug)]
struct WorkstationFile {
	verbs: Vec<Workstation>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Workstation {
	pub id: String,
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
	#[serde(rename = "ID")]
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

fn principles() -> Vec<&'static str> {
	vec!["edge", "forge", "grail", "heart", "knock", "lantern", "moon", "moth", "nectar", "rose", "scale", "sky", "winter"]
}

pub fn init_items(data_path: &PathBuf) -> (HashMap<String, Item>, HashMap<String, Book>, HashMap<String, Skill>, Vec<Workstation>) {
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

	let workstations_rdr = open_data(data_path.clone(), "verbs\\workstations_library_world.json");
	let workstations_json: WorkstationFile = serde_json::from_reader(workstations_rdr).expect("Failed to parse workstations file");

	let wis_rdr = open_data(data_path.clone(), "recipes\\wisdom_commitments.json");
	let wis_json: WisdomCommitments = serde_json::from_reader(wis_rdr).expect("Failed to parse wisdom commitments file");
	let commitments: HashMap<_, _> = wis_json.recipes.into_iter().map(|w| (w.id.clone(), w)).collect();

	let skills_rdr = open_data(data_path.clone(), "elements\\skills.json");
	let skills_json: SkillFile = serde_json::from_reader(skills_rdr).expect("Failed to parse skills file");

	let mut skills = HashMap::new();
	for skill in skills_json.elements {
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

	(items, books, skills, workstations_json.verbs)
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