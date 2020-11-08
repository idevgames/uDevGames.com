// CREATE   /jams/:jam_id/entries               -> jam_entry_id     USERS ONLY
// UPDATE   /jams/:jam_id/entries/:jam_entry_id -> Result<()>       ADMIN/OWNER ONLY
// marking a jam as published is admin-only.
// GET      /jams/:jam_id/:jam_slug/entries     -> Vec<JamEntries>  All when admin,
// GET      /jams/:jam_id/:jam_slug/:jam_entry_id/:jam_entry_slug   otherwise only
//                                              -> Jam              published
// DELETE   /jams/:jam_id/entries/:jam_entry_id -> Result<()>       ADMIN ONLY
