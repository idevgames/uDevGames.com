-- mapping of rich text to attachment. note that this is not intended as a MTM
-- relationship, and that a given attachment ought only ever be owned by one
-- "thing." however, multiple kinds of things can own attachments, so rather
-- than duplicate the attachments table we just have a gentleman's agreement to
-- not cross-pollinate.
CREATE TABLE rich_text_attachments(
    id INTEGER PRIMARY KEY NOT NULL,
    -- the rich text that "owns" this attachment
    rich_text_id INTEGER NOT NULL,
    -- the id of the attachment that the rich text "owns"
    attachment_id INTEGER NOT NULL
);
