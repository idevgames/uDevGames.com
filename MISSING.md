# Missing User Stories

In no particular order:

* `udevgames permission grant -u @login -p foo` has an undefined case. It's
  possible for two users to have the same Github login in our cache. Should
  there be more than one user with the same login this operation should fail,
  and the operator prompted to sync the use records with Github.
* `udevgames user sync -u @login/id` should sync our local user record cache
  with Github, as a remediation step for the above case.
* When I log in I should be automatically redirected to the most recent page I
  was looking at.
* When I have an invalid cookie I should see an error prompting me to log out
  (which will destroy my cookie) and log back in again.
* For some reason the breadcrumbs make the navbar taller? I'd like to to be
  consistent.
* running `cargo run` ought to start brunch watching in a separate process so
  that js and css auto-update without having to re-run commands by hand. note
  that running brunch on `cargo build` is rather past the mark right now as we
  simply run `brunch build --production` in `deploy.sh` instead... which isn't
  fantastic but there's better features to pay into right now.
