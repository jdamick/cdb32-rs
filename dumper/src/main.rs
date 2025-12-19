use std::{io::Result, path::PathBuf};
use std::ffi::CStr;

use cdb32::CDB;

pub fn main() -> Result<()> {
    let flags = xflags::parse_or_exit! {
        /// CDB file path
        required cdb: PathBuf
    };

    let db = CDB::open(flags.cdb)?;

    println!("  {:>40} = value", "key");
    println!("{:->42} - {:->40}", "", "");
    for entry in db.iter() {
        let (key, value) = entry?;
        let keyarr = format!("{:#?}", &key);

        let sk = format!("{:>40}", String::from_utf8(key).unwrap_or(keyarr));

        let strval  = CStr::from_bytes_until_nul(value.as_slice());
        let sv = strval.unwrap().to_string_lossy();

	println!("{:?} = {:?}", sk, sv);
    }

    Ok(())
}
