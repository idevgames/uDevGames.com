
To build, first make sure your system is configured for openssl-sys crate: https://docs.rs/openssl/0.10.29/openssl/

To manage the database migrations install the Diesel CLI.

```bash
sudo apt-get install -y libsqlite3-dev # or equivalent on your os
cargo install diesel_cli --no-default-features --features sqlite
diesel setup
```

rocket mainline branch docs: https://api.rocket.rs/master/rocket/

oauth notes: https://docs.github.com/en/rest/guides/basics-of-authentication

register an application with github: https://github.com/settings/applications/new
name: uDevGames-mysteriouspants
homepage url: https://www.udevgames.com/
description: Periodic game development jams and contests by iDevGames.
callback: http://localhost:4000/gh_callback
