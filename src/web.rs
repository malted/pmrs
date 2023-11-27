#[macro_use]
use rocket::{get, routes, launch, Rocket, Build, State};
use crate::services::Service;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;

#[get("/")]
pub fn index(services: &State<Arc<Vec<Arc<RwLock<Service>>>>>) -> String {
    let mut s = String::new();
    for service in services.iter() {
        s.push_str(&format!(
            "{}: {}\n",
            service.read().name,
            service.read().path.display()
        ));
    }
    s
}

pub async fn rocket(services: Arc<Vec<Arc<RwLock<Service>>>>) -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/", routes![index])
        .manage(services)
        .launch()
        .await?;

    Ok(())
}
