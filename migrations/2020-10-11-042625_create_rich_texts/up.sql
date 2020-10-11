-- rich texts are markdown-formatted longform content that would be found
-- describing game jams, jam entries, development updates on those entries,
-- comments on jam entries or jam entry updates, and describing binary versions
-- of jam entry releases. rich texts can have attachments, which should be
-- limited to images.
CREATE TABLE rich_texts(
    id INTEGER PRIMARY KEY NOT NULL,
    content TEXT NOT NULL
);
