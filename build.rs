use toml_edit::{value, DocumentMut};

fn main() {
    let files = [
        "zkir",
        "base-crypto",
        "coin-structure",
        "ledger",
        "onchain-runtime",
        "onchain-state",
        "onchain-vm",
        "storage",
        "transient-crypto",
        "zswap",
    ];

    for file in files {
        let file_path = format!("./midnight-ledger-prototype/{file}/Cargo.toml");

        let cargo_toml = std::fs::read_to_string(&file_path).unwrap();

        let mut doc = cargo_toml.parse::<DocumentMut>().expect("invalid toml");

        if let Some(deps) = doc.get_mut("dependencies") {
            replace_midnight_dep_with_local(deps);
        }

        if let Some(deps) = doc.get_mut("dev-dependencies") {
            replace_midnight_dep_with_local(deps);
        }

        std::fs::write(file_path, doc.to_string()).unwrap();
    }

    // panic!();
}

fn replace_midnight_dep_with_local(deps: &mut toml_edit::Item) {
    for (key, val) in deps.as_table_like_mut().unwrap().iter_mut() {
        if let Some(dep) = val.as_table_like_mut() {
            let should_replace = dep
                .get("git")
                .map(|dep| {
                    dep.as_str()
                        .unwrap()
                        .starts_with("https://github.com/input-output-hk/midnight-ledger-prototype")
                })
                .unwrap_or(false);

            if should_replace {
                dep.remove("git");
                dep.remove("tag");

                let key = if key == "derive" {
                    "base-crypto-derive".to_string()
                } else if key.starts_with("midnight") {
                    key.to_string()["midnight-".len()..].to_string()
                } else {
                    key.to_string()
                };

                let package = if key.starts_with("midnight") {
                    key.to_string()
                } else {
                    format!("midnight-{key}")
                };

                dep.insert("path", value(format!("../{key}")));
                dep.insert("package", value(package));

                println!("{}", key);

                // println!("{}, {}", &key, &val);
            }
        }
    }
}
