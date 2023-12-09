use clap::Parser;
use color_print::cprintln;
use flack::lock_file;
use pmrs::{caddy, cli, services::Service, SERVICES};
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::{fs::PermissionsExt, net::UnixStream},
    process,
    sync::atomic::Ordering,
    thread,
};

// #[rocket::main]
fn main() -> io::Result<()> {
    match cli::Cli::parse().command {
        cli::Command::Start => start()?,
        cli::Command::Setup => setup()?,
        cli::Command::Status => status()?,
        cli::Command::Daemonise => daemonise()?,
    }

    Ok(())
}

fn start() -> io::Result<()> {
    // Ensure this is the sole instance of pmrs running
    lock_file(&fs::File::open(*pmrs::DEFAULT_CONFIG_PATH)?)?;

    /* Start services */
    {
        for service in SERVICES.iter() {
            thread::spawn(|| Service::spawn(service.clone()));
        }
    }

    /* Web Dashboard */
    {
        rocket::tokio::spawn(async move {
            pmrs::web::rocket()
                .await
                .expect("failed to start the dashboard API")
        });

        if !cfg!(debug_assertions) {
            thread::spawn(|| {
                std::process::Command::new("deno")
                    .arg("run")
                    .arg("--allow-env")
                    .arg("--allow-read")
                    .arg("--allow-net")
                    .arg(*pmrs::DASHBOARD_BUILD_PATH)
                    .spawn()
                    .expect("failed to start web dashboard");
            });
        } else {
            cprintln!("<yellow>Running in debug mode; skipping web dashboard startup</>");
        }
    }

    /* Caddy */
    {
        thread::spawn(|| caddy::start());
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
                thread::sleep(std::time::Duration::from_millis(100));
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

fn setup() -> std::io::Result<()> {
    // Create the config file if it doesn't exist
    if !std::path::Path::new(*pmrs::DEFAULT_CONFIG_PATH).exists() {
        fs::create_dir_all("/etc/pmrs")?;
        fs::copy("src/pmrs.toml.template", *pmrs::DEFAULT_CONFIG_PATH)?;
    }

    // Create the dashboard build directory if it doesn't exist
    if !std::path::Path::new(*pmrs::DASHBOARD_BUILD_PATH).exists() {
        fs::create_dir_all("/usr/share/pmrs/dashboard/build")?;
    }

    Ok(())
}

fn status() -> io::Result<()> {
    let mut stream = UnixStream::connect("/tmp/pmrs.sock")?;
    stream.write_all(b"hello world")?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("{response}");

    Ok(())
}

fn daemonise() -> io::Result<()> {
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
    process::Command::new("systemctl")
        .arg("enable")
        .arg("--now")
        .arg("pmrs.service")
        .spawn()?
        .wait()?;

    Ok(())
}
