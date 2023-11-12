use std::{fs::File, process::Child, sync::Arc, io::Read};
use toml::Table;
use parking_lot::Mutex;
use bus::Bus;
use sysinfo::{System, SystemExt};
use clap::Parser;
use pmrs::services::{Service, spawn_service};
use flack::lock_file;


fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
	let config_file = File::open(pmrs::DEFAULT_CONFIG_PATH)?;

	let cli = pmrs::cli::Cli::parse();
	
	// let mut sys = System::new_all();
	// sys.refresh_all();

	match cli.command {
		pmrs::cli::Command::Start => start(config_file)?,
		pmrs::cli::Command::Status => status(config_file)?,
	}

	Ok(())
}

fn start(mut config_file: File) -> Result<(), Box<dyn std::error::Error + 'static>> {
	// Ensure this is the sole instance of pmrs running
	lock_file(&config_file).expect("another instance of pmrs is already running");

	let mut config_file_buffer = Vec::new();
	config_file.read_to_end(&mut config_file_buffer)?;
	let config: Table = String::from_utf8_lossy(&config_file_buffer).parse()?;

	let services = Service::from_toml(config);

	let bus = Arc::new(Mutex::new(Bus::new(10)));

	// (process reference, restart count)
	let mut children: Vec<(Arc<parking_lot::Mutex<Child>>, usize)> = services.iter().map(|s| (spawn_service(&s, bus.clone(), s.id).expect("failed to spawn service"), 0)).collect::<Vec<_>>();

	let mut rx_web = bus.clone().lock().add_rx();
	let services_web = services.clone();
	std::thread::spawn(move || {
		while let Ok(id) = rx_web.recv() {
			println!("\tPROCESS WITH ID {} FAILED!.", id);
			// let mut file = fs::OpenOptions::new().create(true).append(true).open(format!("logs/{}.log", &services_web[id].name)).expect("failed to open log file");
			// file.write_all(format!("Process with id {} failed\n", id).as_bytes()).expect("failed to write to log file");
		}
	});

	let mut rx = bus.lock().add_rx();
	while let Ok(id) = rx.recv() {
		// Check if the process has failed too many times
		if children[id].1 >= services[id].max_restarts {
			eprintln!("Process with id {} failed for time #{}. It will not be restarted automatically because the max retry count is {}.", id, children[id].1 + 1, services[id].max_restarts);
			continue;
		}
		children[id].1 = children[id].1 + 1;
		println!("Process \"{}\" w/ pmrsid {} failed; restart #{}", &services[id].name, id, children[id].1);
		children[id] = (spawn_service(&services[id], bus.clone(), id)?, children[id].1);
	}

    Ok(())
}

fn status(mut config_file: File) -> Result<(), Box<dyn std::error::Error + 'static>> {
	todo!();
}