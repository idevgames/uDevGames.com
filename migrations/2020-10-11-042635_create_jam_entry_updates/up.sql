-- an update is either some longform text describing some development progress
-- in the entry, or a link to some off-site location describing the same (if
-- someone is going to be kind enough to dedicate some time to our jam the least
-- we can do is offer some promotion to their own personal branding and
-- presence). they may additionally contain binary releases through the rich
-- rich text attachments system. if someone opts to go off-site they cannot also
-- write something that shows up on the site, they need to make another update
-- for that.
CREATE TABLE jam_entry_updates(
    id INTEGER PRIMARY KEY NOT NULL,
    -- the jam entry this is attached to
    jam_entry_id INTEGER NOT NULL,
    -- the title of this update
    title TEXT NOT NULL,
    -- the slug of this update, which is part of the url and makes it easy for
    -- users to see what they're going to read when passing around the url.
    slug TEXT NOT NULL,
    -- a summary text which can appear in a list of updates.
    summary TEXT NOT NULL,
    -- the rich text content this applies to.
    -- should not be present if external_content_url is present.
    rich_text_id INTEGER,
    -- the external content link this applies to.
    -- should not be present if rich_text_id is present.
    external_content_url TEXT,
    -- the approval state, one of "draft" (0), "submitted" (2), "approved" (4),
    -- or "rejected" (8). the approval system is there to prevent malicious
    -- actors from spamming the site.
    approval_state INTEGER NOT NULL DEFAULT 0
);
