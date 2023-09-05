use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::Parser;
use windows::Storage::UserDataPaths;

mod app;
mod data;
mod save;
use save::*;

use crate::data::{init_items, principles_from_soul};

fn main() {
	let args = app::Args::parse();

	let mut data_path = PathBuf::from(args.data_path);
	println!("Using game path: {}", data_path.to_string_lossy());
	data_path.push("bh_Data\\StreamingAssets\\bhcontent\\core");

	let (items, books, skills, workstations) = init_items(&data_path);

	let path = if args.save_path == String::new() {
		println!("Using default save path");
		let mut path = UserDataPaths::GetDefault().expect("Failed to acquire UserDataPaths instance")
			.LocalAppDataLow().expect("Failed to find LocalLow path").to_os_string();
		path.push("\\Weather Factory\\Book of Hours\\AUTOSAVE.json");
		PathBuf::from(path)
	} else {
		PathBuf::from(args.save_path)
	};

	println!("Using save path {}", path.to_string_lossy());

	let save_file = File::open(path).expect("Failed to open save file");
	let save_rdr = BufReader::new(save_file);

	let save: Save = serde_json::from_reader(save_rdr).expect("Failed to parse save file");
	let payloads = save.root_population_command.resolve();


	if args.principle != String::new() {
		println!();
		for payload in payloads {
			for (k, _) in payload.mutations {
				if k.starts_with("mastery") {
					// This is a read book
					let book = match books.get(&payload.entity_id.clone().expect("Book has no entity ID")) {
						Some(book) => book,
						None => continue, // happens for the journal
					};
					let memory = items.get(&book.memory).expect("Couldn't find item for item ID");
					for (aspect, intensity) in &memory.aspects {
						if aspect == &args.principle {
							println!("{} has memory {} with {}: {}", book.label, memory.label, args.principle, intensity)
						}
					}
				}
			}
		}
	} else if args.solve.is_some() {
		if args.level < 1 { panic!("Level should be greater than 0") }
		if args.level > 8 { panic!("Level should be less than 9") }
		let aspects = args.solve.unwrap();
		println!();

		let matching_skills: Vec<_> = skills.clone().into_iter().filter(|(_, s)|
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
					let stations: Vec<_> = workstations.clone().into_iter()
						.filter(|w| w.aspects.contains_key(&wisdom))
						.filter(|w|
							(w.hints.contains(&p1) || w.hints.contains(&p2)) &&
							soul.1.iter().any(|a| w.hints.iter().any(|b| b == a))
						).map(|w| w.label).collect();
					match stations.len() {
						1 => println!(              "{} is upgraded at {} when committed to {}", soul.0, stations[0],      commitment.0.split(".").skip(1).next().unwrap()),
						0 => println!("Warning: {} can't be upgraded with {} when committed to {}", soul.0, skill.1.label, commitment.0.split(".").skip(1).next().unwrap()),
						_ => println!(              "{} is upgraded at {:?} when committed to {}",  soul.0, stations,      commitment.0.split(".").skip(1).next().unwrap()),
					}
				}
				println!();
			}
		}
		println!();

		let read_books = payloads.into_iter()
			.filter(|p| p.mutations.iter().any(|(m, _)| m.starts_with("mastery")))
			.filter_map(|p| books.get(&p.entity_id.expect("Read book has no entity ID")))
			.filter(|b|
				items.get(&b.memory).expect("Couldn't find memory")
					.aspects.iter().any(|a| aspects.iter().any(|arg| arg == a.0))
			);

		let mut rec_books = HashMap::new();
		let memories: HashSet<_> = read_books.clone().map(|b| b.memory.clone()).collect();
		for m in memories {
			rec_books.insert(
				read_books.clone().filter(|b| b.memory == m).next().unwrap().label.clone(),
				items.get(&m).expect("Couldn't find memory").label.clone()
			);
		}

		for (b, m) in rec_books {
			println!("{}: {}", b, m)
		}


	} else if args.aspects.is_some() {
		let mut printed = Vec::new();
		for (_, item) in items {
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
	} else {
		println!("Nothing to do\nUse --help for help")
	}
}
