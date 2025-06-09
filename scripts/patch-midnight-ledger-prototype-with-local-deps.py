#!/usr/bin/env python3

import os
import toml
import pathlib

def main():
    files = [
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
    ]

    for file in files:
        file_path = pathlib.Path(f"./midnight-ledger-prototype/{file}/Cargo.toml")

        with open(file_path, 'r') as f:
            cargo_toml = f.read()

        doc = toml.loads(cargo_toml)

        if "dependencies" in doc:
            replace_midnight_dep_with_local(doc["dependencies"])

        if "dev-dependencies" in doc:
            replace_midnight_dep_with_local(doc["dev-dependencies"])

        with open(file_path, 'w') as f:
            toml.dump(doc, f)

def replace_midnight_dep_with_local(deps):
    for key, val in list(deps.items()):
        if isinstance(val, dict):
            if "git" in val and val.get("git", "").startswith("https://github.com/input-output-hk/midnight-ledger-prototype"):
                val.pop("git", None)
                val.pop("tag", None)

                if key == "derive":
                    key = "base-crypto-derive"
                elif key.startswith("midnight"):
                    key = key[len("midnight-"):]

                package = key
                if not key.startswith("midnight"):
                    package = f"midnight-{key}"

                val["path"] = f"../{key}"
                val["package"] = package

if __name__ == "__main__":
    main()
