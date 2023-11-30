use crate::RUNNING;
use color_print::{cformat, cprint, cprintln};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use toml::Table;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfiguration {
    pub id: usize,
    pub name: String,                // The name of the service.
    pub path: PathBuf,               // A path to the executable file.
    pub args: Vec<String>,           // A list of arguments to pass to the executable file.
    pub envs: Vec<(String, String)>, // A list of kv environment variables to pass to the executable file.
    pub wd: PathBuf, // A path to the working directory from which the executable file should be run.
    pub max_restarts: usize, // The maximum number of times the service can be restarted before pmrs gives up on it. usize::MAX by default.
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

        ServiceConfiguration {
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
			proxy: self.1.get("proxy").map(|i| i.as_str().expect("a str").to_owned()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Service {
	pub configuration: ServiceConfiguration,
	#[serde(skip_serializing, skip_deserializing)]
	pub child: Arc<Option<Child>>,
	pub running: bool,
	pub restarts : usize,
}
impl From<ServiceConfiguration> for Service {
	fn from(configuration: ServiceConfiguration) -> Self {
		Self {
			configuration,
			child: Arc::new(None),
			running: false,
			restarts: 0,
		}
	}
}
impl Service {
	pub fn init(mut config_file: File) -> Result<Arc<Vec<Arc<RwLock<Self>>>>, Box<dyn std::error::Error + 'static>> {
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
		let fmt_service_name = cformat!("<blue, bold>{}</> (id <yellow>{}</>)", s.read().configuration.name, s.read().configuration.id);
		let mut attempts = 0;
		let mut command_successful = false;

		let mut logfile_options = std::fs::OpenOptions::new();
		let logfile_options = logfile_options
			.create(true)
			.write(true)
			.append(true);

		while (!command_successful || s.read().configuration.restart_on_success) && RUNNING.load(Ordering::Relaxed) {
			let cfg = &s.read().configuration;

			let log = logfile_options.open(format!("logs/{}.log", s.read().configuration.name))?;
			let log_err = logfile_options.open(format!("logs/{}.error.log", cfg.name))?;

			let mut child = Command::new(&cfg.path.canonicalize()?)
				.args(&s.read().configuration.args)
				.envs(cfg.envs.clone())
				.current_dir(cfg.wd.canonicalize()?)
				.stdout(log)
				.stderr(log_err)
				.spawn()?;

			attempts += 1;

			match child.wait() {
				Ok(_) if !RUNNING.load(Ordering::Relaxed) => break,
				Ok(output) if output.success() => {
					cprint!(
						"<yellow>Exit #{}</>: {fmt_service_name} successfully exited.",
						attempts
					);
					command_successful = true;
				}
				Ok(_) => {
					cprint!("<red>Failure #{}</>: {fmt_service_name}", attempts);
				}
				Err(_) => {
					cprint!(
						"<red>Failure #{}</> <magenta>(couldn't even start)</>: {fmt_service_name}",
						attempts
					);
				}
			}

			if attempts >= cfg.max_restarts {
				cprintln!(" | <cyan>It will not be restarted automatically.</>");
				break;
			}

			let delay = if cfg.expo_backoff {
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
		s.write().running = false;
		
		Ok(())
	}
}