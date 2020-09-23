-- holds a selection of the data about authenticated github users. really all
-- we care about is the id, to uniquely identify users across github alias
-- changes, their alias as a nicer way to call them, as well as the avatar and
-- profil urls, which may be nice to have later.
CREATE TABLE gh_user_records(
  id BIGINT PRIMARY KEY NOT NULL,
  login TEXT NOT NULL,
  avatar_url TEXT NOT NULL,
  html_url TEXT NOT NULL
);
