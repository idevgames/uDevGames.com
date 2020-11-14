# uDevGames.com

This is the code that runs uDevGames.com, a contest website run by the iDevGames
community.

## Developing

To work on this site, you need Rust.

To build, first make sure your system is configured for openssl-sys crate:
https://docs.rs/openssl/0.10.29/openssl/.

To manage the database migrations install the Diesel CLI.

```bash
sudo apt-get install -y libssl-dev libsqlite3-dev # or equivalent on your os
cargo install diesel_cli --no-default-features --features sqlite
diesel setup
```

uDevGames.com currently uses Rocket's mainline branch to use the unreleased
0.5.x version. As such the documentation you will find on docs.rs is divergent
from what you will find here. You can use https://api.rocket.rs/master/rocket/
to see docs which (mostly) match the reality in the code.

Managing the OAuth lifecycle with Github was done from these reference docs:
https://docs.github.com/en/rest/guides/basics-of-authentication.

To run uDevGames.com locally, first configure an application with Github:
https://github.com/settings/applications/new

**name**: uDevGames-your-github-alias  
**homepage url**: https://www.udevgames.com/  
**description**: Periodic game development jams and contests by iDevGames.  
**callback**: http://localhost:4000/gh_callback  

Note that if you use WSL2 you will have to update that callback to your VM's
current IP address. You can get that address with `hostname -I`. There is a
script called `wsl.sh` which makes this easier, setting the bind address. Call
it with the program arguments you desire, such as `./wsl.sh serve` or
`./wsl.sh migrate`.

Finally, you'll need to configure the application. See `dotenv`, copying that
locally to a `.env` file and filling it in per the instruction in the file.

Happy hacking!

## Project structure

Unless you know better ones.

- `src/models` has a bunch of Rust structs that try to abstract over raw Diesel
  SQL calls. In general the business logic of interacting with the database and
  obeying related constraints is here.
- `src/controllers` has a bunch of servlet-style handlers for individual REST
  routes. They glue together views, models, and view helpers. Try to keep these
  relatively lean.
- `src/template_helpers` is a weird combo of Rust on Rocket
  [request guards](https://rocket.rs/v0.4/guide/requests/#request-guards) that
  can also transform themselves into serializable structs which get fed into
  templates.
- `templates` has a bunch of [tera](https://tera.netlify.app/docs/) templates.
  Tera is a Jinja2 implementation in Rust. In general pages will inherit from
  `layout`.
- `migrations` has a bunch of plain SQL migrations which Diesel can run.
- `static` has a few images and the very important `site.css` file.

The project uses a very weak RBAC authorization implementation by tacking
stringly-typed roles onto user records. Having the string means you have the
role.

Most heavy admin (approving things as public and modifying user roles) is done
through a CLI interface, purely because I was lazy and didn't want to write web
interface for it.

## Modification/Licensing

We want you to be able to use this software regardless of who you may be, what
you are working on, or the environment in which you are working on it - we hope
you'll use it for good and not evil! To this end, the iDevGames website source
code is licensed under the [2-clause BSD][2cbsd] license, with other licenses
available by request. Happy coding!

[2cbsd]: https://opensource.org/licenses/BSD-2-Clause
