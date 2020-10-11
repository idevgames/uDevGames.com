-- the jam entries are the heart of the site, where participants record what
-- they're making and their goals for the jam.
CREATE TABLE jam_entries(
    id INTEGER PRIMARY KEY NOT NULL,
    -- who submitted this entry
    submitter_user_id BIGINT NOT NULL,
    -- the approval state, one of "draft" (0), "submitted" (2), "approved" (4),
    -- or "rejected" (8). the approval system is there to prevent malicious
    -- actors from spamming the site.
    approval_state INTEGER NOT NULL DEFAULT 0,
    -- the text of this entry
    rich_text_id INTEGER NOT NULL
);
