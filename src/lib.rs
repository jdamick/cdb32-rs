//! This crate provides support for reading and writing
//! [CDB](https://cr.yp.to/cdb.html) files. A CDB is a "constant
//! database" that acts as an on-disk associative array mapping keys to
//! values, allowing multiple values for each key. It provides for fast
//! lookups and low overheads. A constant database has no provision for
//! updating, only rewriting from scratch.
//!
//! # Examples
//!
//! Reading a set of records:
//!
//! ```
//! # fn main() -> std::io::Result<()> {
//! use cdb32::CDB;
//!
//! let cdb = CDB::open("tests/test1.cdb")?;
//! for result in cdb.find(b"one") {
//!     println!("{:?}", result?);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Creating a database with safe atomic updating:
//!
//! ```
//! # fn main() -> std::io::Result<()> {
//! # let tmp_dir = tempfile::tempdir()?;
//! # let tmp_path = tmp_dir.path();
//! # std::env::set_current_dir(&tmp_path)?;
//! use cdb32::CDBWriter;
//!
//! let mut cdb = CDBWriter::create("temporary.cdb")?;
//! cdb.add(b"one", b"Hello, ")?;
//! cdb.add(b"one", b"world!\n")?;
//! cdb.add(b"two", &[1, 2, 3, 4])?;
//! cdb.finish()?;
//! # Ok(())
//! # }
//! ```
//!
//! # References
//!
//!  * [D. J. Bernstein's original software](https://cr.yp.to/cdb.html)
//!  * [Constant Database (cdb) Internals](https://www.unixuser.org/~euske/doc/cdbinternals/index.html)
//!  * [Wikipedia](https://en.wikipedia.org/wiki/Cdb_(software))

mod hash;
mod reader;
mod uint32;
mod writer;

pub use crate::reader::{CDBIter, CDBKeyValueIter, CDBValueIter, Result, CDB};
pub use crate::writer::{CDBMake, CDBWriter};
