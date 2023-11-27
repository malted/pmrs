use crate::RUNNING;
use bus::Bus;
use color_print::{cformat, cprint, cprintln};
use parking_lot::{Mutex, RwLock};
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use toml::Table;

#[derive(Deserialize, Debug, Clone)]
pub struct Service {
    pub id: usize,
    pub name: String,                // The name of the service.
    pub path: PathBuf,               // A path to the executable file.
    pub args: Vec<String>,           // A list of arguments to pass to the executable file.
    pub envs: Vec<(String, String)>, // A list of kv environment variables to pass to the executable file.
    pub wd: PathBuf, // A path to the working directory from which the executable file should be run.
    pub max_restarts: usize, // The maximum number of times the service can be restarted before pmrs gives up on it. usize::MAX by default.
    pub restart_on_success: bool, // Whether or not to restart the service when it exits successfully.
    pub expo_backoff: bool, // Whether or not to use exponential backoff when restarting the service.

                            // Nonconfigurables
                            // pub child: Option<Child>,
                            // pub running: Arc<AtomicBool>,
}
impl Service {
    pub fn from_toml(config: Table) -> Vec<Self> {
        config
            .get("services")
            .unwrap_or(&toml::Value::Array(vec![]))
            .as_table()
            .unwrap_or(&toml::map::Map::new())
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let mut s: Service = s.into();
                s.id = idx;

                // Add global envs
                config
                    .get("envs")
                    .unwrap()
                    .as_table()
                    .unwrap()
                    .iter()
                    .for_each(|(key, value)| {
                        s.envs
                            .push((key.to_owned(), value.as_str().expect("a str").to_owned()));
                    });

                s
            })
            .collect()
    }
}

pub type ServiceEntry<'a> = (&'a String, &'a toml::Value);
impl Into<Service> for ServiceEntry<'_> {
    fn into(self) -> Service {
        let name = self.0.to_owned();
        let path: PathBuf = self
            .1
            .get("path")
            .expect("a path")
            .as_str()
            .expect("")
            .into();
        if !path.is_file() {
            panic!(
                "The supplied path ({}) for service \"{name}\" is not a file. It should be.",
                path.display()
            )
        }

        let wd: PathBuf = if let Some(wd) = self
            .1
            .get("wd")
            .and_then(|x| x.as_str())
            .map(|x| Into::<PathBuf>::into(x))
        {
            if !wd.is_dir() {
                panic!("The supplied working directory ({}) for service \"{name}\" is not a directory. It should be.", wd.display())
            }
            wd
        } else if self
            .1
            .get("auto_wd")
            .map(|x| x.as_bool().unwrap_or(false))
            .unwrap_or(false)
        {
            path.as_path().parent().expect("a parent").to_owned()
        } else {
            env::current_dir().expect("a current dir")
        };

        Service {
            id: usize::MAX,
            name,
            path,
            args: self
                .1
                .get("args")
                .map(|i| i.as_array().expect("an array"))
                .unwrap_or(&vec![])
                .iter()
                .map(|i| i.as_str().expect("a str").to_owned())
                .collect(),
            envs: self
                .1
                .get("envs")
                .map(|i| i.as_table().expect("a table"))
                .unwrap_or(&toml::map::Map::new())
                .iter()
                .map(|(key, value)| (key.to_owned(), value.as_str().expect("a str").to_owned()))
                .collect(),
            wd,
            max_restarts: self
                .1
                .get("max_restarts")
                .map(|i| {
                    i.as_integer()
                        .expect("the max restart count to be an integer")
                        as usize
                })
                .unwrap_or(usize::MAX),
            restart_on_success: self
                .1
                .get("restart_on_success")
                .map(|i| i.as_bool().expect("a bool"))
                .unwrap_or(true),
            expo_backoff: self
                .1
                .get("expo_backoff")
                .map(|i| i.as_bool().expect("a bool"))
                .unwrap_or(false),
        }
    }
}

// Spawns a program
// Funnels its stdout and stderr to a log file
// If it fails, broadcasts to a Bus
// Returns a handle to the process

pub fn spawn_service(
    service: Arc<RwLock<Service>>,
    services_killed: Arc<Mutex<Vec<bool>>>,
) -> std::io::Result<()> {
    let fmt_service_name = cformat!(
        "<blue, bold>{}</> (id <yellow>{}</>)",
        service.read().name,
        service.read().id
    );

    let mut attempts = 0;
    let mut command_successful = false;

    while (!command_successful || service.read().restart_on_success)
        && RUNNING.load(Ordering::Relaxed)
    {
        let servlk = service.read();

        let log = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(format!("logs/{}.log", servlk.name))?;
        let log_err = log.try_clone()?;

        let mut child = Command::new(&servlk.path.canonicalize()?)
            .args(&servlk.args)
            .envs(servlk.envs.clone())
            .current_dir(servlk.wd.canonicalize()?)
            .stdout(log)
            .stderr(log_err)
            .spawn()?;

        attempts += 1;

        drop(servlk);

        match child.wait() {
            Ok(_) if !RUNNING.load(Ordering::Relaxed) => break,
            Ok(output) if output.success() => {
                cprint!(
                    "<yellow>Exit #{}</>: {fmt_service_name} successfully exited.",
                    attempts
                );
                command_successful = true;
            }
            Ok(output) => {
                cprint!("<red>Failure #{}</>: {fmt_service_name}", attempts);
            }
            Err(e) => {
                cprint!(
                    "<red>Failure #{}</> <magenta>(couldn't even start)</>: {fmt_service_name}",
                    attempts
                );
            }
        }

        if attempts >= service.read().max_restarts {
            cprintln!(" | <cyan>It will not be restarted automatically.</>");
            break;
        }

        let delay = if service.read().expo_backoff {
            2u64.pow(attempts as u32)
        } else {
            1
        };

        if delay > 0 {
            cprintln!(" | <cyan>Restarting in {} seconds</>", delay);
            std::thread::sleep(std::time::Duration::from_secs(delay));
        } else {
            cprintln!(" | <cyan>Restarting immediately</>");
        }
    }

    cprintln!("<magenta>Termination</>: {fmt_service_name}.");
    services_killed.lock()[service.read().id] = true;

    Ok(())
}
