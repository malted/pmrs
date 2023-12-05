use clap::Parser;
use color_print::cprintln;
use flack::lock_file;
use pmrs::{caddy, services::Service, SERVICES};
use std::sync::atomic::Ordering;
use std::{
    fs::File,
    io::{Read, Write},
};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    match pmrs::cli::Cli::parse().command {
        pmrs::cli::Command::Start => start()?,
        pmrs::cli::Command::Status => status()?,
        pmrs::cli::Command::Daemonise => daemonise()?,
    }

    Ok(())
}

fn start() -> Result<(), Box<dyn std::error::Error + 'static>> {
    // Ensure this is the sole instance of pmrs running
    lock_file(&File::open(*pmrs::DEFAULT_CONFIG_PATH)?)?;

    /* Start services */
    {
        for service in SERVICES.iter() {
            cprintln!(
                "<green>Starting</> <blue, bold>{}</>",
                service.read().configuration.name
            );
            std::thread::spawn(move || Service::spawn(service.clone()));
        }
    }

    /* Web Dashboard */
    {
        rocket::tokio::spawn(async move {
            pmrs::web::rocket()
                .await
                .expect("failed to start the dashboard API")
        });
        // std::thread::spawn(|| {
        //     std::process::Command::new("deno")
        //         .arg("run")
        //         .arg("--allow-env")
        //         .arg("--allow-read")
        //         .arg("--allow-net")
        //         .arg(*pmrs::DASHBOARD_BUILD_PATH)
        //         .spawn()
        //         .expect("failed to start web dashboard");
        // });
    }

    /* Caddy */
    {
        std::thread::spawn(|| caddy::start());
    }

    /* Graceful shutdown */
    {
        ctrlc::set_handler(move || {
            cprintln!("\n<red>Stopping</> <blue, bold>pmrs</>");
            pmrs::RUNNING.store(false, Ordering::SeqCst);
            // Wait until all services are killed.
            // If the below panic occurs, it means the service was not killed, or there is a zombie ID.
            let mut i = 0;
            while &SERVICES.iter().any(|s| s.read().running) == &true {
                std::thread::sleep(std::time::Duration::from_millis(100));
                i += 1;
                if i > 50 {
                    panic!("Failed to stop all services in 5 seconds. Please file a bug!");
                }
            }
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

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
}

fn status() -> Result<(), Box<dyn std::error::Error + 'static>> {
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
