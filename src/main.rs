#![feature(str_from_utf16_endian)]

use std::{
	collections::{HashMap, HashSet},
	fs::File,
	io::BufReader,
	path::PathBuf,
};

use clap::Parser;

mod app;
mod data;
mod logic;
mod save;

use data::{init_items, principles_from_soul};
use logic::find_memories;
use save::{default_save_path, Save};

fn main() {
	let args = app::Args::parse();

	let mut data_path = PathBuf::from(args.data_path);
	println!("Using game path: {}", data_path.to_string_lossy());
	data_path.push("StreamingAssets");
	data_path.push("bhcontent");
	data_path.push("core");
	let data = init_items(&data_path);

	let path = if args.save_path == String::new() {
		println!("Using default save path");
		default_save_path()
	} else {
		PathBuf::from(args.save_path)
	};
	println!("Using save path {}", path.to_string_lossy());
	let save_file = File::open(path).expect("Failed to open save file");
	let save_rdr = BufReader::new(save_file);
	let save: Save = serde_json::from_reader(save_rdr).expect("Failed to parse save file");
	let (save_items, skills, _abilities) = save.resolve();
	let world_items = save_items.into_iter().filter(|i|
		data.items.contains_key(&i.id) ||
		data.books.contains_key(&i.id) ||
		data.skills.contains_key(&i.id)).collect();

	if args.principle != String::new() {
		println!();
		for mem in find_memories(&[&args.principle], 8, &world_items, &data.items, &data.books) {
			let intensity = mem.aspects.get(&args.principle).unwrap();
			println!("{} has memory {} with {}: {}", mem.source_label, mem.label, args.principle, intensity)
		}
	} else if args.solve.is_some() {
		let aspects = args.solve.unwrap();
		println!();
		let matching_skills: Vec<_> = data.skills.clone().into_iter().filter(|(_, s)|
			(s.principles.0 == aspects[0] || s.principles.0 == aspects[1]) &&
			(s.principles.1 == aspects[0] || s.principles.1 == aspects[1])
		).collect();
		if matching_skills.len() == 0 {
			println!("Warning: no matching skills")
		} else {
			println!("Matching skills:");
			for skill in matching_skills {
				println!("{}", skill.1.label);
				let (p1, p2) = (skill.1.principles.0, skill.1.principles.1);
				for commitment in vec![skill.1.wisdoms.0, skill.1.wisdoms.1] {
					let soul = principles_from_soul(&commitment.1);
					let wisdom = "e.".to_string() + commitment.0.split(".").skip(1).next().unwrap();
					let stations: Vec<_> = data.workstations.clone().into_iter()
						.filter(|w| w.aspects.contains_key(&wisdom))
						.filter(|w|
							(w.hints.contains(&p1) || w.hints.contains(&p2)) &&
							soul.1.iter().any(|a| w.hints.iter().any(|b| b == a))
						).map(|w| w.label).collect();
					match stations.len() {
						1 => println!(                 "{} is upgraded at {} when committed to {}", soul.0, stations[0],   commitment.0.split(".").skip(1).next().unwrap()),
						0 => println!("Warning: {} can't be upgraded with {} when committed to {}", soul.0, skill.1.label, commitment.0.split(".").skip(1).next().unwrap()),
						_ => println!(               "{} is upgraded at {:?} when committed to {}", soul.0, stations,      commitment.0.split(".").skip(1).next().unwrap()),
					}
				}
				println!();
			}
		}
		println!();

		for mem in find_memories(&[&aspects[0], &aspects[1]], 8, &world_items, &data.items, &data.books) {
			println!("{}:\t {}", mem.source_label, mem.label)
		}
	} else if args.aspects.is_some() {
		let mut printed = Vec::new();
		for (_, item) in data.items {
			let label = if item.label.starts_with("Lepidoptery") || item.label.starts_with("Wire") {
				item.label
			} else {
				item.label.split("(").next().unwrap().to_owned()
			};
			if printed.contains(&label) {
				continue;
			}
			let mut valid = true;
			for aspect in args.aspects.clone().unwrap() {
				if !item.aspects.contains_key(&aspect) {
					valid = false;
				}
			}
			if valid {
				println!();
				println!("{}", &label);
				printed.push(label);
				for (aspect, intensity) in item.aspects {
					if !aspect.starts_with("boost") {
						print!("{aspect}: {intensity}  	");
					}
				}
			}
		}
	} else if args.craft.is_some() {
		println!();
		let craft = args.craft.unwrap();

		let known_recipes: HashSet<_> = data.recipes.0.iter().chain(data.recipes.1.iter()).chain(data.recipes.2.iter())
			.filter(|r| skills.contains(&r.skill)).map(|r| (r.label.clone(), r.principle.clone(), r.ingredient.clone())).collect();

		if data::principles().contains(&craft.as_str()) {

		} else {
			let skill = match data.skills.iter().filter(|(_, s)| s.label.to_lowercase().starts_with(&craft.to_lowercase())).next() {
				Some(s) => s,
				None => { println!("Skill not found: {}", craft); return; },
			};
			println!("Using skill {}\n", skill.1.label);

			println!("Prentice recipes:");
			for r in data.recipes.0.into_iter().filter(|r| skill.0 == &r.skill) {
				match known_recipes.get(&(r.label.clone(), r.principle.clone(), r.ingredient.clone())) {
					Some(_) => println!("{} ({})", r.label, r.principle),
					None    => println!("{} ({}) [New Recipe!]", r.label, r.principle),
				}
			}

			println!();
			println!("Scholar recipes:");
			for r in data.recipes.1.into_iter().filter(|r| skill.0 == &r.skill) {
				let item = r.ingredient.clone().expect("Scholar recipe uses no ingredient");
				match known_recipes.get(&(r.label.clone(), r.principle.clone(), r.ingredient.clone())) {
					Some(_) => println!("{} using {} ({})", r.label, item, r.principle),
					None    => println!("{} using {} ({}) [New Recipe!]", r.label, item, r.principle),
				}
			}

			println!();
			println!("Keeper recipes:");
			for r in data.recipes.2.into_iter().filter(|r| skill.0 == &r.skill) {
				let item = data.items.get(&r.ingredient.clone().expect("Keeper recipe uses no ingredient")).expect("Couldn't find ingredient");
				match known_recipes.get(&(r.label.clone(), r.principle.clone(), r.ingredient.clone())) {
					Some(_) => println!("{} using {} ({})", r.label, item.label, r.principle),
					None    => println!("{} using {} ({}) [New Recipe!]", r.label, item.label, r.principle),
				}
			}
		}
	} else {
		println!("Nothing to do\nUse --help for help")
	}
}
