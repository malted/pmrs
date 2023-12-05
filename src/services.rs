use crate::RUNNING;
use color_print::{cformat, cprint, cprintln};
use parking_lot::RwLock;
use rocket::tokio::io::stdout;
use serde::{Deserialize, Serialize};
use std::env::{self, args};
use std::fs::File;
use std::io::Read;
use std::io::Stdout;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use toml::Table;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfiguration {
    pub id: usize,
    pub name: String,                // The name of the service.
    pub args: Vec<String>,           // A list of arguments to pass to the executable file.
    pub envs: Vec<(String, String)>, // A list of kv environment variables to pass to the executable file.
    pub wd: PathBuf, // A path to the working directory from which the executable file should be run.
    pub cmd: String, // If the cmd is a valid path, treat it as a binary. Otherwise, treat it as a shell command.
    pub max_restarts: Option<usize>, // The maximum number of times the service can be restarted before pmrs gives up on it. None by default.
    pub restart_on_success: bool, // Whether or not to restart the service when it exits successfully.
    pub expo_backoff: bool, // Whether or not to use exponential backoff when restarting the service.
    pub proxy: Option<String>, // Proxy the service through this url root.
}
impl ServiceConfiguration {
    pub fn from_toml(config: Table) -> Vec<Self> {
        config
            .get("services")
            .unwrap_or(&toml::Value::Array(vec![]))
            .as_table()
            .unwrap_or(&toml::map::Map::new())
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let mut s: Self = s.into();
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

pub type ServiceConfigurationEntry<'a> = (&'a String, &'a toml::Value);
impl Into<ServiceConfiguration> for ServiceConfigurationEntry<'_> {
    fn into(self) -> ServiceConfiguration {
        ServiceConfiguration {
            id: usize::MAX,
            name: self.0.to_owned(),
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
            wd: self
                .1
                .get("wd")
                .map(|i| {
                    PathBuf::from(i.as_str().expect("a str"))
                        .canonicalize()
                        .unwrap()
                })
                .unwrap_or(env::current_dir().unwrap())
                .to_owned()
                .into(),
            cmd: self
                .1
                .get("cmd")
                .map(|x| x.as_str().expect("a str").to_owned())
                .expect("a cmd key"),
            max_restarts: self
                .1
                .get("max_restarts")
                .map(|i| {
                    Some(
                        i.as_integer()
                            .expect("the max restart count to be an integer")
                            as usize,
                    )
                })
                .unwrap_or(None),
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
            proxy: self
                .1
                .get("proxy")
                .map(|i| i.as_str().expect("a str").to_owned()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
    pub configuration: ServiceConfiguration,
    pub running: bool,
    pub restarts: usize,
    pub exit_code: Option<i32>,
}
impl From<ServiceConfiguration> for Service {
    fn from(configuration: ServiceConfiguration) -> Self {
        Self {
            configuration,
            running: false,
            restarts: 0,
            exit_code: None,
        }
    }
}
impl Service {
    pub fn init(
        mut config_file: File,
    ) -> Result<Arc<Vec<Arc<RwLock<Self>>>>, Box<dyn std::error::Error + 'static>> {
        let mut config_file_buffer = Vec::new();
        config_file.read_to_end(&mut config_file_buffer)?;
        let config: Table = String::from_utf8_lossy(&config_file_buffer).parse()?;

        let services: Vec<Arc<RwLock<Service>>> = ServiceConfiguration::from_toml(config)
            .iter()
            .map(|s| Arc::new(RwLock::new(Service::from(s.clone()))))
            .collect();

        Ok(Arc::new(services))
    }

    /// Spawn a service.
    ///
    /// Spawn a service with the given configuration. The service will be spawned in a new thread.
    /// Funnel its stdout and stderr to a log file.
    ///
    pub fn spawn(s: Arc<RwLock<Self>>) -> std::io::Result<()> {
        let fmt_service_name = cformat!(
            "<blue, bold>{}</> (id <yellow>{}</>)",
            s.read().configuration.name,
            s.read().configuration.id
        );
        let mut attempts = 0;
        let mut command_successful = false;

        let mut logfile_options = std::fs::OpenOptions::new();
        let logfile_options = logfile_options.create(true).write(true).append(true);

        while (!command_successful || s.read().configuration.restart_on_success)
            && RUNNING.load(Ordering::Relaxed)
        {
            attempts += 1;
            cprintln!("<green>Attempt #{}</>: {fmt_service_name}", attempts);

            let log = logfile_options.open(format!("logs/{}.log", s.read().configuration.name))?;
            let log_err =
                logfile_options.open(format!("logs/{}.error.log", s.read().configuration.name))?;

            let program = match PathBuf::from(&s.read().configuration.cmd).canonicalize() {
                Ok(canonical_path) if canonical_path.is_file() => canonical_path
                    .to_str()
                    .map(|s| s.to_owned())
                    .expect("path is not a vaild utf-8 string"),
                _ => s.read().configuration.cmd.clone(),
            };

            let mut program = program.split_whitespace();

            let mut child =
                std::process::Command::new(&program.next().expect("a program name/path"))
                    .args(
                        program
                            .clone()
                            .map(|s| s.to_string())
                            .chain(s.read().configuration.args.clone())
                            .collect::<Vec<String>>(),
                    )
                    .envs(s.read().configuration.envs.clone())
                    .current_dir("dashboard")
                    .stdout(log)
                    .stderr(log_err)
                    .spawn()?;

            s.write().running = true;

            match child.wait() {
                Ok(_) if !RUNNING.load(Ordering::Relaxed) => break,
                Ok(output) if output.success() => {
                    cprint!(
                        "<yellow>Exit #{}</>: {fmt_service_name} successfully exited.",
                        attempts
                    );
                    command_successful = true;
                    s.write().running = false;
                }
                Ok(_) => {
                    cprint!("<red>Failure #{}</>: {fmt_service_name}", attempts);
                    s.write().running = false;
                }
                Err(_) => {
                    cprint!(
                        "<red>Failure #{}</> <magenta>(couldn't even start)</>: {fmt_service_name}",
                        attempts
                    );
                    s.write().running = false;
                }
            }

            // Failures is attempts - 1 because the first attempt is not a failure.
            if let Some(max_restarts) = s.read().configuration.max_restarts {
                if attempts > max_restarts {
                    cprintln!(" | <cyan>It will not be restarted automatically.</>");
                    break;
                }
            }

            let delay = if s.read().configuration.expo_backoff {
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

            s.write().restarts += 1;
        }

        s.write().running = false;

        cprintln!("<magenta>Termination</>: {fmt_service_name}.");

        Ok(())
    }
}
