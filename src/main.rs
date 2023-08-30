use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use clap::Parser;
use windows::Storage::UserDataPaths;

mod items;
mod save;
use save::*;

use crate::items::init_items;

#[derive(Parser, Debug)]
struct Args {
	/// Path to save file
	#[arg(short, long, default_value_t = String::new())]
	save_path: String,

	/// Path to game data
	#[arg(short, long)]
	data_path: String,

	/// Principle of the memory you are looking for
	#[arg(short, long)]
	principle: String,
}

fn main() {
	let args = Args::parse();

	let mut data_path = PathBuf::from(args.data_path);
	println!("Using game path: {}", data_path.to_string_lossy());
	data_path.push("bh_Data\\StreamingAssets\\bhcontent\\core");

	let (items, books) = init_items(&data_path);

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
						println!("{} has memory {} with {}: {}", book.label, book.memory, args.principle, intensity)
					}
				}
			}
		}
	}
}
