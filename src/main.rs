#![feature(str_from_utf16_endian, map_try_insert, string_remove_matches)]

use std::{
	env::args, fs, path::PathBuf, sync::{Arc, Mutex}, time::Duration
};

use axum::{routing::get, Router};
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

mod app;
mod data;
mod logic;
mod save;
mod ui;

use app::*;
use data::init_items;
use save::{default_save_path, SaveData};

#[tokio::main]
async fn main() {
	let mut data_path = PathBuf::from(args().nth(1).expect("Data path required"));
	data_path.extend(["StreamingAssets", "bhcontent", "core"]);
	println!("Using game path: {}", data_path.to_string_lossy());
	let data = init_items(&data_path);

	let path = default_save_path();
	println!("Using save path {}", path.to_string_lossy());
	let mut save = SaveData::from_path(path.clone());
	save.items = save.items.into_iter().filter(|i|
		data.items.contains_key(&i.id) ||
		data.books.contains_key(&i.id) ||
		data.skills.contains_key(&i.id)).collect();

	let state = AppState {
		data: Arc::new(data),
		save: Arc::new(Mutex::new(save)),
	};

	let save = state.save.clone();
	let data = state.data.clone();
	tokio::spawn(async move {
		let mut modified = fs::metadata(&path).and_then(|m| m.modified()).expect("Unable to init save metadata");
		loop {
			sleep(Duration::from_secs(10)).await;
			if let Ok(time) = fs::metadata(&path).and_then(|m| m.modified()) {
				if modified != time {
					modified = time;
					let mut new_save = SaveData::from_path(path.clone());
					new_save.items = new_save.items.into_iter().filter(|i|
						data.items.contains_key(&i.id) ||
						data.books.contains_key(&i.id) ||
						data.skills.contains_key(&i.id)).collect();
					let mut s = save.lock().unwrap();
					*s = new_save;
					drop(s);
				}
			}
		}
	});

	let app = Router::new()
		.route("/", get(root))
		.route("/find_mems", get(p_form).post(find_mems))
		.route("/solve", get(s_form).post(solve))
		.route("/crafting", get(c_form).post(crafting))
		.route("/items", get(i_form).post(items))
		.nest_service("/assets",
			ServiceBuilder::new()
			.service(ServeDir::new("assets")))
		.with_state(state);
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
		.await
		.unwrap();
	axum::serve(listener, app).await.unwrap();
}
