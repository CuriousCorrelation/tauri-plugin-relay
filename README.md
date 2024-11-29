# Tauri Plugin Hoppscotch Relay

A HTTP request-response relay plugin for Tauri applications for advanced request handling including custom headers, certificates, proxies, and local system integration. Used by Hoppscotch Desktop and Hoppscotch Agent.

## Install

_This plugin requires a Rust version of at least **1.77.2**_

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-hoppscotch-relay = { git = "https://github.com/CuriousCorrelation/tauri-plugin-hoppscotch-relay" }
```

Install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add https://github.com/CuriousCorrelation/tauri-plugin-hoppscotch-relay
# or 
npm add https://github.com/CuriousCorrelation/tauri-plugin-hoppscotch-relay
# or
yarn add https://github.com/CuriousCorrelation/tauri-plugin-hoppscotch-relay
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_hoppscotch_relay::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Then you can use the plugin in your JavaScript/TypeScript code:

```typescript
import { run, cancel } from 'tauri-plugin-hoppscotch-relay-api'

// Make a request
const response = await run({
  req: {
    req_id: 1,
    method: 'GET',
    endpoint: 'https://api.example.com',
    headers: [],
    validate_certs: true
  }
})

// Cancel a request
await cancel({ req_id: 1 })
```

## Features

- HTTP/HTTPS request handling with custom headers
- Support for various request body types (Text, URL-encoded, Form-data)
- Client certificate support (PEM and PFX formats)
- Custom root certificate bundles
- Proxy configuration
- Request cancellation
- Detailed response metadata including timing information

## License

Code: (c) 2024 - CuriousCorrelation

MIT or MIT/Apache 2.0 where applicable.
