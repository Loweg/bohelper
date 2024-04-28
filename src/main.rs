#![feature(str_from_utf16_endian)]

use std::{
	collections::HashSet,
	fs::File,
	io::BufReader,
	path::PathBuf,
};

use clap::Parser;

mod app;
mod data;
mod logic;
mod save;

use data::init_items;
use logic::{find_memories, print_skill_stations};
use save::{default_save_path, Save};

use crate::logic::print_aspected;

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
		for (mem, (source, aspects)) in find_memories(&[&args.principle], 8, &world_items, &data.items, &data.books) {
			let intensity = aspects.get(&args.principle).unwrap();
			println!("{} has memory {} with {}: {}", source, mem, args.principle, intensity)
		}
	} else if args.solve.is_some() {
		let aspects = args.solve.unwrap();
		println!();

		let matching_skills: Vec<_> = data.skills.iter()
			.filter(|(_, s)| s.matches_exact(&aspects))
			.map(|(_, s)| s).collect();

		if matching_skills.is_empty() {
			println!("Warning: no matching skills")
		} else {
			println!("Matching skills:");
			print_skill_stations(&matching_skills, &data.workstations);
		}
		println!();

		for (mem, (source, _)) in find_memories(&[&aspects[0], &aspects[1]], 8, &world_items, &data.items, &data.books) {
			println!("{}:\t {}", source, mem)
		}
	} else if args.aspects.is_some() {
		print_aspected(&data.items, &args.aspects.unwrap())
	} else if args.craft.is_some() {
		println!();
		let craft = args.craft.unwrap();

		let known_recipes: HashSet<_> = data.recipes.0.iter().chain(data.recipes.1.iter()).chain(data.recipes.2.iter())
			.filter(|r| skills.contains(&r.skill)).map(|r| r.label.clone()).collect();

		if data::principles().contains(&craft.as_str()) {

		} else {
			let skill = match data.skills.iter()
				.find(|(_, s)| s.label.to_lowercase().starts_with(&craft.to_lowercase()))
			{
				Some(s) => s,
				None => { println!("Skill not found: {}", craft); return; },
			};
			println!("Using skill {}\n", skill.1.label);

			println!("Prentice recipes:");
			for r in data.recipes.0.into_iter().filter(|r| skill.0 == &r.skill) {
				match known_recipes.get(&r.label) {
					Some(_) => println!("{} ({})", r.label, r.principle),
					None    => println!("{} ({}) [New Recipe!]", r.label, r.principle),
				}
			}

			println!();
			println!("Scholar recipes:");
			for r in data.recipes.1.into_iter().filter(|r| skill.0 == &r.skill) {
				let item = r.ingredient.clone().expect("Scholar recipe uses no ingredient");
				match known_recipes.get(&r.label) {
					Some(_) => println!("{} using {} ({})", r.label, item, r.principle),
					None    => println!("{} using {} ({}) [New Recipe!]", r.label, item, r.principle),
				}
			}

			println!();
			println!("Keeper recipes:");
			for r in data.recipes.2.into_iter().filter(|r| skill.0 == &r.skill) {
				let item = data.items.get(&r.ingredient.clone().expect("Keeper recipe uses no ingredient")).expect("Couldn't find ingredient");
				match known_recipes.get(&r.label) {
					Some(_) => println!("{} using {} ({})", r.label, item.label, r.principle),
					None    => println!("{} using {} ({}) [New Recipe!]", r.label, item.label, r.principle),
				}
			}
		}
	} else {
		println!("Nothing to do\nUse --help for help")
	}
}
