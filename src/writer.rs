use std::{
    cmp::max,
    ffi::OsString,
    fs,
    io::{self, prelude::*, Result},
    iter,
    path::PathBuf,
};

use crate::{hash::hash, uint32};

#[derive(Clone, Copy, Debug)]
struct HashPos {
    hash: u32,
    pos: u32,
}

impl HashPos {
    fn pack(&self, buf: &mut [u8]) {
        uint32::pack2(buf, self.hash, self.pos);
    }
}

fn err_toobig<T>() -> Result<T> {
    Err(io::Error::new(io::ErrorKind::Other, "File too big"))
}

/// Base interface for making a CDB file.
///
/// # Example
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # let tmp_dir = tempfile::tempdir()?;
/// # let tmp_path = tmp_dir.path();
/// # std::env::set_current_dir(&tmp_path)?;
/// let file = std::fs::File::create("temporary.cdb")?;
/// let mut cdb = cdb32::CDBMake::new(file)?;
/// cdb.add(b"one", b"Hello,")?;
/// cdb.add(b"two", b"world!")?;
/// cdb.finish()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CDBMake {
    entries: Vec<Vec<HashPos>>,
    pos: u32,
    file: io::BufWriter<fs::File>,
}

impl CDBMake {
    /// Create a new CDB maker.
    pub fn new(file: fs::File) -> Result<CDBMake> {
        let mut w = io::BufWriter::new(file);
        let buf = [0; 2048];
        w.seek(io::SeekFrom::Start(0))?;
        w.write_all(&buf)?;
        Ok(CDBMake {
            entries: iter::repeat(vec![]).take(256).collect::<Vec<_>>(),
            pos: 2048,
            file: w,
        })
    }

    fn pos_plus(&mut self, len: u32) -> Result<()> {
        if self.pos + len < len {
            err_toobig()
        } else {
            self.pos += len;
            Ok(())
        }
    }

    fn add_end(&mut self, keylen: u32, datalen: u32, hash: u32) -> Result<()> {
        self.entries[(hash & 0xff) as usize].push(HashPos {
            hash,
            pos: self.pos,
        });
        self.pos_plus(8)?;
        self.pos_plus(keylen)?;
        self.pos_plus(datalen)?;
        Ok(())
    }

    fn add_begin(&mut self, keylen: u32, datalen: u32) -> Result<()> {
        let mut buf = [0; 8];
        uint32::pack2(&mut buf[0..8], keylen, datalen);
        self.file.write_all(&buf)?;
        Ok(())
    }

    /// Add a record to the CDB file.
    pub fn add(&mut self, key: &[u8], data: &[u8]) -> Result<()> {
        if key.len() >= 0xffffffff || data.len() >= 0xffffffff {
            return Err(io::Error::new(io::ErrorKind::Other, "Key or data too big"));
        }
        self.add_begin(key.len() as u32, data.len() as u32)?;
        self.file.write_all(key)?;
        self.file.write_all(data)?;
        self.add_end(key.len() as u32, data.len() as u32, hash(key))
    }

    /// Set the permissions on the underlying file.
    pub fn set_permissions(&self, perm: fs::Permissions) -> Result<()> {
        self.file.get_ref().set_permissions(perm)
    }

    /// Finish writing to the CDB file and flush its contents.
    pub fn finish(mut self) -> Result<()> {
        let mut buf = [0; 8];

        let maxsize = self.entries.iter().fold(1, |acc, e| max(acc, e.len() * 2));
        let count = self.entries.iter().fold(0, |acc, e| acc + e.len());
        if maxsize + count > (0xffffffff / 8) {
            return err_toobig();
        }

        let mut table = vec![HashPos { hash: 0, pos: 0 }; maxsize];

        let mut header = [0_u8; 2048];
        for i in 0..256 {
            let len = self.entries[i].len() * 2;
            let j = i * 8;
            uint32::pack2(&mut header[j..j + 8], self.pos, len as u32);

            for e in self.entries[i].iter() {
                let mut wh = (e.hash as usize >> 8) % len;
                while table[wh].pos != 0 {
                    wh += 1;
                    if wh == len {
                        wh = 0;
                    }
                }
                table[wh] = *e;
            }

            for hp in table.iter_mut().take(len) {
                hp.pack(&mut buf);
                self.file.write_all(&buf)?;
                self.pos_plus(8)?;
                *hp = HashPos { hash: 0, pos: 0 };
            }
        }

        self.file.flush()?;
        self.file.seek(io::SeekFrom::Start(0))?;
        self.file.write_all(&header)?;
        self.file.flush()?;
        Ok(())
    }
}

/// A CDB file writer which handles atomic updating.
///
/// Using this type, a CDB file is safely written by first creating a
/// temporary file, building the CDB structure into that temporary file,
/// and finally renaming that temporary file over the final file name.
/// If the temporary file is not properly finished (ie due to an error),
/// the temporary file is deleted when this writer is dropped.
///
/// # Example
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # let tmp_dir = tempfile::tempdir()?;
/// # let tmp_path = tmp_dir.path();
/// # std::env::set_current_dir(&tmp_path)?;
/// use cdb32::CDBWriter;
///
/// let mut cdb = CDBWriter::create("temporary.cdb")?;
/// cdb.add(b"one", b"Hello")?;
/// cdb.finish()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CDBWriter {
    dstname: PathBuf,
    tmpname: PathBuf,
    cdb: Option<CDBMake>,
}

impl CDBWriter {
    /// Safely create a new CDB file.
    ///
    /// The suffix for the temporary file defaults to `".tmp"`.
    pub fn create<P: Into<PathBuf>>(filename: P) -> Result<CDBWriter> {
        CDBWriter::with_suffix(filename, ".tmp")
    }

    /// Safely create a new CDB file, using a specific suffix for the temporary file.
    pub fn with_suffix<P: Into<PathBuf>>(filename: P, suffix: &str) -> Result<CDBWriter> {
        let filename = filename.into();
        let mut tmpname = filename.clone();
        let new_extension = match tmpname.extension() {
            Some(ext) => {
                let mut ext = ext.to_os_string();
                ext.push(suffix);
                ext
            }
            None => OsString::from(suffix),
        };
        tmpname.set_extension(new_extension);
        CDBWriter::with_filenames(filename, &tmpname)
    }

    /// Safely create a new CDB file, using two specific file names.
    ///
    /// Note that the temporary file name must be on the same filesystem
    /// as the destination, or else the final rename will fail.
    pub fn with_filenames<P: Into<PathBuf>, Q: Into<PathBuf>>(
        filename: P,
        tmpname: Q,
    ) -> Result<CDBWriter> {
        let dstname = filename.into();
        let tmpname = tmpname.into();
        let file = fs::File::create(&tmpname)?;
        let cdb = CDBMake::new(file)?;
        Ok(CDBWriter {
            dstname,
            tmpname,
            cdb: Some(cdb),
        })
    }

    /// Add a record to the CDB file.
    pub fn add(&mut self, key: &[u8], data: &[u8]) -> Result<()> {
        // The unwrap() is safe here, as the internal cdb is only ever
        // None during finish(), which does not call this.
        self.cdb.as_mut().unwrap().add(key, data)
    }

    /// Set permissions on the temporary file.
    ///
    /// This must be done before the file is finished, as the temporary
    /// file will no longer exist at that point.
    pub fn set_permissions(&self, perm: fs::Permissions) -> Result<()> {
        self.cdb.as_ref().unwrap().set_permissions(perm)
    }

    pub fn finish(mut self) -> Result<()> {
        self.cdb.take().unwrap().finish()?;
        fs::rename(&self.tmpname, &self.dstname)?;
        Ok(())
    }
}

impl Drop for CDBWriter {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        if self.cdb.is_some() {
            fs::remove_file(&self.tmpname);
        }
    }
}
