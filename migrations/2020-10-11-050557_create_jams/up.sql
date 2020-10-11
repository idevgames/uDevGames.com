-- a jam, which is a kind of contest or public forcing function to get people to
-- work on their video games. it's fun! they run for a specific duration and may
-- have specific themes.
CREATE TABLE jams(
    id INTEGER PRIMARY KEY NOT NULL,
    -- text describing the jam, it's goals, themes, rules, etc.
    rich_text_id INTEGER NOT NULL,
    -- when does the jam "start." most of these are super informal and if you
    -- get a head start people will actually be happy for you.
    start_date,
    -- the end date, a totally artificial and contrived end date designed to
    -- help you gamify your own productivity into pushing just a little harder.
    end_date,
    -- the approval state, one of "draft" (0), "submitted" (2), "approved" (4),
    -- or "rejected" (8). the approval system is there to prevent malicious
    -- actors from spamming the site, but in this case is here to allow the
    -- creation of draft jams without prematurely putting something on the
    -- frontpage.
    approval_state INTEGER NOT NULL DEFAULT 0
);
