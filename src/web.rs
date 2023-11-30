use rocket::{get, routes, State};
use crate::services::Service;
use parking_lot::RwLock;
use std::sync::Arc;
use rocket::serde::json::Json;
use crate::SERVICES;

#[get("/")]
pub fn index() -> String {
	println!("workinrst");
    format!("Hello, world!")
}

#[get("/services")]
pub fn services() -> Json<Vec<Service>> {
	// Services to json
	println!("lengh {:#?}", SERVICES.len());
	Json(SERVICES.clone().iter().map(|s| {
		let s = s.read().clone();
		println!("{:?}", s);
		s
	}).collect())
}

#[get("/test")]
pub fn test() -> String {
	let mut fin = String::new();
	fin += SERVICES[0].read().configuration.name.as_str();
	format!("There are {} services\n{}", SERVICES.len(), fin)
}

pub async fn rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index, services, test])
        .manage(services)
        .launch()
        .await?;

    Ok(())
}
