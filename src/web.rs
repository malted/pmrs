use rocket::{get, routes};
use crate::services::Service;
use rocket::serde::json::Json;
use crate::SERVICES;

#[get("/")]
pub fn index() -> String {
    format!("Hello, world!")
}

#[get("/services")]
pub fn services() -> Json<Vec<Service>> {
	Json(SERVICES.iter().map(|s| s.read().clone()).collect())
}

pub async fn rocket() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index, services])
        .manage(services)
        .launch()
        .await?;

    Ok(())
}
