use crate::PORT_CADDY;
use parking_lot::RwLock;
use rocket::http::hyper::uri::Port;
use std::sync::Arc;
use std::{collections::HashMap, io::{self, Write}};

use crate::services::Service;

pub fn start() -> io::Result<()> {
    // For now, the configuation will be loaded in the Caddyfile format.
    // If you somehow understand Caddy's JSON configuation schema, please open a PR :P
    let mut services_by_domain: HashMap<String, Vec<Arc<RwLock<Service>>>> = HashMap::new();
    for service in crate::SERVICES.iter() {
        if let Some(ref mut proxy) = service.read().configuration.proxy.clone() {
            if crate::HTTP_RE.find(&proxy).is_none() {
                proxy.insert_str(0, "http://");
            } else {
                panic!("Proxies should not contain the protocol (http:// or https://)");
            }

            let url = url::Url::parse(&proxy).expect(&format!("failed to parse {proxy}"));
            let domain = url
                .domain()
                .expect("the proxy should just be a domain")
                .to_owned();

            if let Some(t) = services_by_domain.get_mut(&domain) {
                t.push(service.to_owned());
            } else {
                services_by_domain.insert(domain, vec![service.clone()]);
            }
        }
    }

    // Now build the Caddyfile!
    let mut caddyfile = String::new();
    for (domain, services) in services_by_domain.iter() {
        caddyfile.push_str(&format!("{domain} {{"));
 
        for service in services.iter() {
            let conf = &service.read().configuration;
            if let (Some(proxy), Some(port)) = (&conf.proxy, conf.port) {
                let url =
                    url::Url::parse(&format!("http://{proxy}")).expect("could not parse proxy url");
                let path = url.path();

                // Use tab: <	>
                caddyfile.push_str(&format!(r"
	rewrite {path}/ {path}
	route {path} {{
		uri strip_prefix {path}
		reverse_proxy localhost:{port}
	}}
"));
            }
        }

        // Finally, add the dashboard
        caddyfile.push_str(&format!(r"
	rewrite /admin/ /admin
	route /admin {{
		uri strip_prefix /admin
		reverse_proxy localhost:3000
	}}
"));

        caddyfile.push_str("}\n");
    }
    println!("{caddyfile}"); 
    let client = reqwest::blocking::Client::new();
    
    let upload_response = client.post("http://localhost:2019/load")
        .header("Content-Type", "text/caddyfile")
        .body(caddyfile)
        .send()
        .expect("coud not esnd caddyfile")
        .text()
        .expect("i wanted text :(((");

    println!("caddyresponse: {upload_response}");

    Ok(())
}
