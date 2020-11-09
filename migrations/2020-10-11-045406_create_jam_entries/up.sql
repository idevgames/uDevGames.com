-- the jam entries are the heart of the site, where participants record what
-- they're making and their goals for the jam.
CREATE TABLE jam_entries(
    id INTEGER PRIMARY KEY NOT NULL,
    -- who submitted this entry
    submitter_user_id BIGINT NOT NULL,
    -- the approval state, one of "draft" (0), "submitted" (2), "approved" (4),
    -- or "rejected" (8). the approval system is there to prevent malicious
    -- actors from spamming the site.
    approval_state INTEGER CHECK(approval_state IN (0, 1, 2, 5, 8)) NOT NULL DEFAULT 0,
    -- the title of this jam entry
    title TEXT NOT NULL,
    -- the slug of this jam entry, which is part of the url and makes it easy
    -- for users to see what they're going to read when passing around the url.
    slug TEXT NOT NULL,
    -- a summary text which can appear in a list of jame entries.
    summary TEXT NOT NULL,
    -- summary image, which is an attachment.
    summary_attachment_id INTEGER NOT NULL,
    -- the text of this entry
    rich_text_id INTEGER NOT NULL
);
