use clap::Parser;
use color_print::{cprint, cprintln};
use flack::lock_file;
use parking_lot::{Mutex, RwLock};
use pmrs::services::{spawn_service, Service};
use rocket::tokio::{
    self, task,
    time::{sleep, Duration},
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::unix::net::UnixListener,
    path::Path,
    process::Child,
    sync::Arc,
};
use sysinfo::{System, SystemExt};
use toml::Table;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    // Setup signal handling
    // let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");
    // let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");

    let config_file = File::open(pmrs::DEFAULT_CONFIG_PATH)?;

    let cli = pmrs::cli::Cli::parse();

    // let mut sys = System::new_all();
    // sys.refresh_all();

    match cli.command {
        pmrs::cli::Command::Start => start(config_file)?,
        pmrs::cli::Command::Status => status(config_file)?,
        pmrs::cli::Command::Daemonise => daemonise()?,
    }

    Ok(())
}

fn start(mut config_file: File) -> Result<(), Box<dyn std::error::Error + 'static>> {
    // Ensure this is the sole instance of pmrs running
    lock_file(&config_file).expect("another instance of pmrs is already running");

    let mut config_file_buffer = Vec::new();
    config_file.read_to_end(&mut config_file_buffer)?;
    let config: Table = String::from_utf8_lossy(&config_file_buffer).parse()?;

    // let services: Vec<RwLock<Service>> = Service::from_toml(config)
    //     .into_iter()
    //     .map(RwLock::new)
    //     .collect();
    let services: Arc<Vec<Arc<RwLock<Service>>>> = Arc::new(
        Service::from_toml(config)
            .into_iter()
            .map(|s| Arc::new(RwLock::new(s)))
            .collect(),
    );

    let highest_id = services.iter().map(|s| s.read().id).max().unwrap_or(0);
    let services_killed = Arc::new(Mutex::new(vec![false; highest_id + 1])); // Maybe this should be a hashmap to avoid zombie IDs?

    // (process reference, init count)
    for service in services.iter() {
        cprintln!("<green>Starting</> <blue, bold>{}</>", service.read().name);

        let service_clone = service.clone();
        let sk_clone = services_killed.clone();
        std::thread::spawn(move || spawn_service(service_clone, sk_clone));
    }

    rocket::tokio::spawn(async move {
        pmrs::web::rocket(services.clone())
            .await
            .expect("run web dashboard.")
    });

    let ctrlc_sk = services_killed.clone();
    ctrlc::set_handler(move || {
        cprintln!("\n<red>Stopping</> <blue, bold>pmrs</>");
        pmrs::RUNNING.store(false, Ordering::SeqCst);
        // Wait until all services are killed.
        // If the below panic occurs, it means the service was not killed, or there is a zombie ID.
        let mut i = 0;
        while ctrlc_sk.lock().iter().any(|&x| x == false) {
            std::thread::sleep(std::time::Duration::from_millis(100));
            i += 1;
            if i > 100 {
                panic!("Failed to stop all services in 5 seconds. Please file a bug!");
            }
        }
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    loop {}

    // let sock_listener = UnixListener::bind("/tmp/pmrs.sock2")?;
    // match sock_listener.accept() {
    //     Ok((mut socket, addr)) => {
    //         println!("Got a client: {:?} - {:?}", socket, addr);
    //         socket.write_all(b"hello world").expect("wriet tio string");
    //         let mut response = String::new();
    //         socket
    //             .read_to_string(&mut response)
    //             .expect("reda to string");
    //         println!("{}", response);
    //     }
    //     Err(e) => println!("accept function failed: {:?}", e),
    // }

    Ok(())
}

fn status(mut config_file: File) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut stream = std::os::unix::net::UnixStream::connect("/tmp/pmrs.sock")?;
    stream.write_all(b"hello world")?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{response}");

    Ok(())
}

use std::fs;
use std::os::unix::fs::PermissionsExt;
fn daemonise() -> std::io::Result<()> {
    let service_file = if cfg!(debug_assertions) {
        "testing/systemd/pmrs.service"
    } else {
        "/etc/systemd/system/pmrs.service"
    };
    fs::copy("src/systemd/pmrs.service.template", service_file)?;

    // Set permissions
    let metadata = fs::metadata(&service_file)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(service_file, permissions)?;

    // Reload systemd
    std::process::Command::new("systemctl")
        .arg("enable")
        .arg("--now")
        .arg("pmrs.service")
        .spawn()?
        .wait()?;

    Ok(())
}
