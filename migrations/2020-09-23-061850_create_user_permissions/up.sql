-- permissions have a name, and they're rather stringly typed,
-- so having the "admin" permission makes you an admin, having
-- the "Admin" permission would not. Poor design, but it doesn't
-- need to be particularly robust, either.
CREATE TABLE permissions(
    id INTEGER PRIMARY KEY NOT NULL,
    gh_user_id BIGINT NOT NULL,
    name TEXT NOT NULL
);
