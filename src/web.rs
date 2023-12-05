use crate::services::Service;
use crate::{SERVICES, sysinfo_wrappers};
use rocket::serde::json::Json;
use rocket::{get, routes, State};
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};
use parking_lot::RwLock;
use std::sync::Arc;
use serde_json::json;
use serde::Serialize;
use rocket_ws as ws;

fn system_internal(sys_info: &State<RwLock<System>>) -> sysinfo_wrappers::System {
	sys_info.write().refresh_all();
	sys_info.write().refresh_networks();
	sys_info.write().refresh_disks();

	sysinfo_wrappers::System::init(sys_info.inner())
}
fn services_internal() -> Vec<Service> {
	SERVICES.iter().map(|s| s.read().clone()).collect()
}


#[get("/")]
pub fn index() -> String {
    format!("Hello, world!")
}

#[get("/system")]
pub fn system(sys_info: &State<RwLock<System>>) -> Json<sysinfo_wrappers::System> {
	Json(system_internal(sys_info))
}

#[get("/services")]
pub fn services() -> Json<Vec<Service>> {
	Json(services_internal())
}

#[get("/ws")]
fn websocket<'a>(sys_info: &'a State<RwLock<System>>, ws: ws::WebSocket) -> ws::Stream!['a] {
    ws::Stream! { ws =>
        for await message in ws {
			if let Ok(ws::Message::Text(ref text)) = message {
				if text == "ping" {
					yield ws::Message::Text("pong".to_string());
				} else if text == "system" {
					yield ws::Message::Text(json!(system_internal(sys_info)).to_string());
				} else if text == "services" {
					yield ws::Message::Text(json!(services_internal()).to_string());
				}
			}
        }
    }
}

pub async fn rocket() -> Result<(), rocket::Error> {
	let sys_info = RwLock::new(System::new_all());

    let _rocket = rocket::build()
        .mount("/", routes![index, system, services, websocket])
        .manage(services)
		.manage(sys_info)
        .launch()
        .await?;

    Ok(())
}
