use bus::Bus;
use clap::Parser;
use color_print::{cprint, cprintln};
use flack::lock_file;
use parking_lot::Mutex;
use pmrs::services::{spawn_service, Service, ServiceRepr};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    process::Child,
    sync::Arc,
};
use sysinfo::{System, SystemExt};
use toml::Table;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
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

    let services = Service::from_toml(config);

    let bus = Arc::new(Mutex::new(Bus::new(100)));

    // (process reference, init count)
    let mut children: Vec<ServiceRepr> = services
        .iter()
        .map(|s| {
            cprintln!("<green>Starting</> <blue, bold>{}</>", s.name.clone());
            (
                spawn_service(&s, bus.clone(), s.id, 0).expect("failed to spawn"),
                0,
            )
        })
        .collect::<Vec<ServiceRepr>>();

    let mut rx_web = bus.clone().lock().add_rx();
    let children_web = children.clone();
    let services_web = services.clone();
    std::thread::spawn(move || {
        while let Ok(id) = rx_web.recv() {

            // let mut file = fs::OpenOptions::new().create(true).append(true).open(format!("logs/{}.log", &services_web[id].name)).expect("failed to open log file");
            // file.write_all(format!("Process with id {} failed\n", id).as_bytes()).expect("failed to write to log file");
        }
    });

    let mut rxx = bus.clone().lock().add_rx();
    std::thread::spawn(move || {
        while let Ok(id) = rxx.recv() {
            cprint!(
                "<red>Failure #{}: </><blue,bold>{}</> (id <yellow>{id}</>)",
                children[id].1 + 1,
                services[id].name,
            );

            // Check if the process has failed too many times
            if children[id].1 >= services[id].max_restarts {
                cprintln!(
                    " | <cyan>It will not be restarted automatically because the max retry count is {}</>",
                    services[id].max_restarts
                );
                continue;
            }

            let delay = if services[id].expo_backoff {
                2u64.pow(children[id].1 as u32) + services[id].restart_delay
            } else {
                services[id].restart_delay
            };

            if delay > 0 {
                cprintln!(" | <cyan>Restarting in {} seconds</>", delay);
            } else {
                cprintln!(" | <cyan>Restarting immediately</>");
            }

            children[id] = (
                spawn_service(&services[id], bus.clone(), id, delay).unwrap(),
                children[id].1 + 1,
            );
        }
    });

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
