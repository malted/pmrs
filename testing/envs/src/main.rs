use std::io::Write;

fn main() {
	let envs = std::env::vars();

	let mut file = std::fs::File::create("envs.txt").expect("create failed");
	for (key, value) in envs {
		let s = format!("{}={}\n", key, value);
		file.write_all(s.as_bytes()).expect("write failed");
	}
}

