use toml::Table;
use serde::Deserialize;
use std::path::PathBuf;
use std::env;

#[derive(Deserialize, Debug, Clone)]
pub struct Service {
	pub id: usize,
	pub name: String, // The name of the service.
	pub path: PathBuf, // A path to the executable file.
	pub args: Vec<String>, // A list of arguments to pass to the executable file.
	pub envs: Vec<(String, String)>, // A list of kv environment variables to pass to the executable file.
	pub wd: PathBuf,  // A path to the working directory from which the executable file should be run.
	pub max_restarts: usize, // The maximum number of times the service can be restarted before pmrs gives up on it.
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
				config.get("envs").unwrap().as_table().unwrap().iter().for_each(|(key, value)| {
					s.envs.push((key.to_owned(), value.as_str().expect("a str").to_owned()));
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
		let path: PathBuf = self.1.get("path").expect("a path").as_str().expect("").into();
		if !path.is_file() { panic!("The supplied path ({}) for service \"{name}\" is not a file. It should be.", path.display()) }

		let wd: PathBuf = if let Some(wd) = self.1.get("wd").and_then(|x| x.as_str()).map(|x| Into::<PathBuf>::into(x)) {
			if !wd.is_dir() { panic!("The supplied working directory ({}) for service \"{name}\" is not a directory. It should be.", wd.display()) }
			wd
		} else if self.1.get("auto_wd").map(|x| x.as_bool().unwrap_or(false)).unwrap_or(false) {
			path.as_path().parent().expect("a parent").to_owned()
		} else {
			env::current_dir().expect("a current dir")
		};

		Service {
			id: usize::MAX,
			name,
			path,
			args: self.1.get("args").map(|i| i.as_array().expect("an array")).unwrap_or(&vec![]).iter().map(|i| i.as_str().expect("a str").to_owned()).collect(),
			envs: self.1.get("envs").map(|i| i.as_table().expect("a table")).unwrap_or(&toml::map::Map::new()).iter().map(|(key, value)| (key.to_owned(), value.as_str().expect("a str").to_owned())).collect(),
			wd,
			max_restarts: self.1.get("max_restarts").map(|i| i.as_integer().expect("the max restart count to be an integer") as usize).unwrap_or(crate::DEFAULT_MAX_RESTARTS),
		}
	}
}

use std::process::{Command, Child, Stdio};
use std::sync::Arc;
use parking_lot::Mutex;
use bus::Bus;

pub fn spawn_service(service: &Service, fail_tx: Arc<Mutex<Bus<usize>>>, id: usize) -> std::io::Result<Arc<Mutex<Child>>> {
	let child = Command::new(&service.path.canonicalize()?)
            .args(&service.args)
            .envs(service.envs.clone())
            .current_dir(service.wd.canonicalize()?)
			.stdout(Stdio::piped())
            .spawn()?;

	let child_arc = Arc::new(Mutex::new(child));

	let child_for_thread = Arc::clone(&child_arc);
    std::thread::spawn(move || {
        let mut child = child_for_thread.lock();
        let status = child.wait().expect("failed to wait on child");

        if !status.success() { fail_tx.lock().broadcast(id) }
    });

	Ok(child_arc)
}