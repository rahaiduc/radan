# Radan

My personal portfolio site. Built with a Rust backend and a static frontend, all wrapped in a One Piece pirate theme because why not.

https://radan-production.up.railway.app/

The server is written with [Axum](https://github.com/tokio-rs/axum) and handles a simple but useful layout system: there is a single `.layout.html` file that acts as the shell, and each page is just a fragment that gets injected into it at the `{{outlet}}` placeholder.

## Stack

- **Backend:** Rust, Axum, Tokio
- **Frontend:** Plain HTML and CSS, no framework
- **Container:** Docker with a multi-stage build, so the final image only contains the compiled binary and the web folder

## Project structure

```
radan/
  server/       Rust source code and Cargo files
  web/          Frontend files (HTML fragments, CSS, assets)
  Dockerfile
```

## About

I am Raul, a developer from Spain focused on backend, cybersecurity, AI agents, and automation. This repo is the ship I sail on. Every other repo is an island.