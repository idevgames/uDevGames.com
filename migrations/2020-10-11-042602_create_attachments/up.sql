-- attachments are files that can be served for download.
CREATE TABLE attachments(
    id INTEGER PRIMARY KEY NOT NULL,
    -- the path to the file on disk
    file TEXT NOT NULL,
    -- the name of the file, for example as shown in the rich text editor
    name TEXT NOT NULL,
    -- the mime type of the file, which will be set on download so that images
    -- behave like images, etc.
    mime_type TEXT NOT NULL,
    -- the md5 of the file. this could be validated periodically to prevent us
    -- from vending compromised files, and shown to users downloading binaries
    -- with an encouragement to validate the md5 of what they have downloaded.
    md5 BLOB NOT NULL
);
