pub fn start() {
    std::process::Command::new("caddy")
        .arg("run")
        .spawn()
        .expect("failed to start caddy");

    let caddy_port = crate::PORT_CADDY.to_string();
    let client = reqwest::blocking::Client::new();

    let existing_config = client
        .get(format!("http://localhost:{caddy_port}/config/"))
        .send()
        .expect("failed to send request")
        .text()
        .expect("failed to read response");

    println!("{}", existing_config);

    let req = client
        .post(format!("http://localhost:{caddy_port}/load"))
        .json(&serde_json::json!({
            "apps": {
                "http": {
                    "servers": {
                        "hello": {
                            "listen": [":2015"],
                            "routes": [
                                {
                                    "handle": [{
                                        "handler": "static_response",
                                        "body": "Hello, world!"
                                    }]
                                }
                            ]
                        }
                    }
                }
            }
        }))
        .send()
        .expect("failed to send request");
}
