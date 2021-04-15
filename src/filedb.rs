use rusqlite::{OptionalExtension, ToSql};
use std::io::Write;

/// A handle to a file database on the local file system.
///
/// Currently, this type refers to an SQLITE3 database underneath. The binary text data from the
/// file is stored in the database rows in a compressed format. This is probably NOT a good way to
/// store large files.
pub struct FileDB {
    conn: rusqlite::Connection,
}

impl FileDB {
    /// Connect to a file database stored at the provided path.
    pub fn connect<P: AsRef<std::path::Path>>(path: P) -> Result<Self, crate::error::Error> {
        let conn = rusqlite::Connection::open(path.as_ref())?;
        conn.execute(DB_INIT_QUERY, rusqlite::NO_PARAMS)?;

        Ok(Self { conn })
    }

    /// Retrieve a file from the database.
    ///
    /// # Arguments
    ///
    /// * key - Typically a file name, but it can be any string really. It does not need to be
    /// unique for each file in the database.
    /// * time_stamp - This is the time stamp you want associated with this entry. This library
    /// will not automatically generate time stamps, it is up to the user to decide how they want
    /// time stamps generated.
    ///
    /// # Returns
    ///
    /// It returns the content of the file in a buffer. If the database entry was `NULL`, then it
    /// returns `None` in the option. If it can't find a file with the correct key and time stamp
    /// then it will return an error.
    pub fn retrieve_file(
        &self,
        key: &str,
        time_stamp: chrono::NaiveDateTime,
    ) -> Result<Option<Vec<u8>>, crate::error::Error> {
        let mut writer = Vec::new();
        let mut deflater = flate2::write::ZlibDecoder::new(writer);

        let time_stamp: i64 = time_stamp.timestamp();

        let bytes: Option<Vec<u8>> = self
            .conn
            .query_row(
                DB_RETRIEVE_FILE_QUERY,
                &[&key as &dyn ToSql, &time_stamp as &dyn ToSql],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(bytes) = bytes {
            deflater.write_all(&bytes[..])?;
            writer = deflater.finish()?;
            Ok(Some(writer))
        } else {
            Ok(None)
        }
    }

    /// List all files in the database.
    ///
    /// # Returns
    /// 
    /// Returns an interator of tuples with the key and timestamp of all the files in the 
    /// archive.
    pub fn list_all(&'_ self) -> Result<Vec<(String, chrono::NaiveDateTime)>, crate::error::Error> {

        let mut stmt = self.conn.prepare("SELECT key, time_stamp FROM files")?;

        let all = stmt.
            query_map(rusqlite::NO_PARAMS, |row| row.get(0).and_then(|key| row.get(1).map(|ts| (key, ts))))?
            .filter_map(|res| res.ok())
            .map(|(key, ts)| (key, chrono::NaiveDateTime::from_timestamp(ts, 0))).collect();

        Ok(all)

    }

    /// Add a file to the database.
    ///
    /// # Arguments
    ///
    /// * key - Typically a file name, but it can be any string really. It does not need to be
    /// unique for each file in the database.
    /// * time_stamp - This is the time stamp you want associated with this entry. This library
    /// will not automatically generate time stamps, it is up to the user to decide how they want
    /// time stamps generated.
    /// * data - Is the file contents to store in the database.
    ///
    pub fn add_file(
        &self,
        key: &str,
        time_stamp: chrono::NaiveDateTime,
        data: &[u8],
    ) -> Result<(), crate::error::Error> {
        let compressed_data: Vec<u8> = Vec::with_capacity(data.len());
        let mut encoder =
            flate2::write::ZlibEncoder::new(compressed_data, flate2::Compression::default());
        encoder.write_all(data)?;
        let compressed_data = encoder.finish()?;

        let time_stamp: i64 = time_stamp.timestamp();

        self.conn.execute(
            DB_INSERT_FILE_QUERY,
            &[
                &key as &dyn ToSql,
                &time_stamp as &dyn ToSql,
                &compressed_data as &dyn ToSql,
            ],
        )?;

        Ok(())
    }
}

impl Drop for FileDB {
    fn drop(&mut self) {
        let earliest_date = chrono::offset::Utc::now().naive_utc() - chrono::Duration::days(365);
        let _ = self.conn.execute(DB_CLEANUP_QUERY, &[earliest_date]);
    }
}

const DB_INIT_QUERY: &'static str = include_str!("init_query.sql");
const DB_RETRIEVE_FILE_QUERY: &'static str = include_str!("retrieve_file.sql");
const DB_INSERT_FILE_QUERY: &'static str = include_str!("insert_query.sql");
const DB_CLEANUP_QUERY: &'static str = include_str!("cleanup_query.sql");

#[cfg(test)]
mod test {
    use std::io::Read;

    #[test]
    fn test_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        let temp_db = tempfile::NamedTempFile::new_in(".").unwrap();
        let db_fname = temp_db.path();
        let db = super::FileDB::connect(db_fname).unwrap();

        let time_stamp = chrono::offset::Utc::now().naive_utc();
        let mut test_data: String = String::new();
        let mut test_data_file = std::fs::File::open("src/filedb.rs").unwrap();
        test_data_file.read_to_string(&mut test_data).unwrap();

        db.add_file("filedb.rs", time_stamp, test_data.as_bytes())
            .unwrap();

        let bytes = db.retrieve_file("filedb.rs", time_stamp).unwrap().unwrap();
        let retrieved = String::from_utf8(bytes).unwrap();

        assert_eq!(test_data, retrieved);

        Ok(())
    }

    #[test]
    fn test_no_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_db = tempfile::NamedTempFile::new_in(".").unwrap();
        let db_fname = temp_db.path();
        let db = super::FileDB::connect(db_fname).unwrap();

        let time_stamp = chrono::offset::Utc::now().naive_utc();
        let mut test_data: String = String::new();
        let mut test_data_file = std::fs::File::open("src/filedb.rs").unwrap();
        test_data_file.read_to_string(&mut test_data).unwrap();

        db.add_file("filedb.rs", time_stamp, test_data.as_bytes())
            .unwrap();

        let bytes = db
            .retrieve_file("some_file_that_is_not_in_there", time_stamp)
            .unwrap();
        assert!(bytes.is_none());

        Ok(())
    }
}
