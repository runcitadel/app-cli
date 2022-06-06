# Citadel app CLI

This is a new tool to convert Citadel app.yml files to a custom format that can be further processed by a Python script.

The main goal is to move most of the app.yml parsing into Docker, and have less code on the host.

### Example usage

`cargo run -- convert app.yml result.yml --port-map ports.json --services "lnd"`
`cargo run -- convert samourai.yml result.yml --port-map ports.json --services "lnd"`

### Get a jsonschema

`cargo run -- schema`
