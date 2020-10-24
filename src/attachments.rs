use md5::compute as md5compute;
use std::{
    fs::{ read as read_file, rename, },
    io::Error as IOError,
    path::PathBuf,
};
use thiserror::Error;


#[derive(Debug, Error)]
pub enum AttachmentStorageError {
    #[error("IOError {0}")]
    IoError(#[from] IOError),

    #[error("File does not exist at path {0}")]
    NotFound(PathBuf),
}

/// Hides the logic of placing files into storage and retrieving them again.
pub struct AttachmentStorage {
    /// Where the files are stored.
    storage_path: PathBuf,
}

/// The stored attachment itself.
pub struct StoredAttachment {
    /// Where the file itself is stored.
    path: PathBuf,
    /// The md5sum, lazily computed on retrieval.
    md5sum: Option<[u8; 16]>,
}

impl AttachmentStorage {
    /// Creates a new attachment storage managing the given storage path.
    pub fn new(storage_path: PathBuf) -> AttachmentStorage {
        AttachmentStorage { storage_path }
    }

    /// Stores the file with the given attachment id. The original file is
    /// consumed.
    pub fn store(
        &self, file: &PathBuf, attachment_id: i32
    ) -> Result<StoredAttachment, AttachmentStorageError> {
        // place the file in the store by id
        let destination_path: PathBuf = {
            let mut path = self.storage_path.clone();
            path.push(attachment_id.to_string());
            path
        };

        rename(file, &destination_path)?;

        // compute the md5 - we do this on store so the attachments db can
        // immediately be updated with the md5.
        let mut stored_attachment = StoredAttachment {
            path: destination_path, md5sum: None
        };
        stored_attachment.get_or_compute_md5()?;
        
        Ok(stored_attachment)
    }

    /// Loads an attachment out of storage, returning a File for it.
    pub fn load(
        &self, attachment_id: i32
    ) -> Result<std::fs::File, AttachmentStorageError> {
        let storage_path = {
            let mut path = self.storage_path.clone();
            path.push(attachment_id.to_string());
            path
        };

        if !storage_path.exists() || !storage_path.is_file() {
            return Err(AttachmentStorageError::NotFound(storage_path));
        }

        Ok(std::fs::File::open(storage_path)?)
    }
}

impl StoredAttachment {
    pub fn get_or_compute_md5(
        &mut self
    ) -> Result<[u8; 16], AttachmentStorageError> {
        match self.md5sum {
            None => {
                let md5 = md5_file(&self.path)?;
                self.md5sum = Some(md5);
                Ok(md5)
            },
            Some(md5sum) => Ok(md5sum)
        }
    }
}

// TODO: digest as a stream, rather than reading the whole mess into memory
// may need a different crate for that :(
fn md5_file(file: &PathBuf) -> Result<[u8; 16], IOError> {
    let contents = read_file(&file)?;
    let digest = md5compute(&contents);
    let sum: &[u8] = digest.as_ref();
    let mut buffer: [u8; 16] = [0; 16];
    buffer.copy_from_slice(&sum);
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use crate::attachments::*;
    use hex::decode as hex_decode;
    use std::io::Write;

    /// This may look dumb to test... and it kind of is. But I wanted to make
    /// sure the digester was working in a way consistent with expectation. So I
    /// wrote a test for it. It didn't make sense to delete it.
    #[test]
    fn test_file_hashing() {
        let example_content = "this is an example";
        let expected_sum = 
            hex_decode("9202816dabaaf34bb106a10421b9a0d0").unwrap();
        let file = tempfile::NamedTempFile::new().unwrap();
        write!(&file, "{}", example_content).unwrap();
        let actual_sum = md5_file(&file.path().to_path_buf()).unwrap();
        assert_eq!(expected_sum, actual_sum);
    }
}
