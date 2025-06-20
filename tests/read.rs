use std::fs;

use cdb2::{CDBWriter, CDB};

#[test]
fn test_one() {
    let cdb = CDB::open("tests/test1.cdb").unwrap();
    let mut i = cdb.find(b"one");
    assert_eq!(i.next().unwrap().unwrap(), b"Hello");
    assert_eq!(i.next().unwrap().unwrap(), b", World!");
}

#[test]
fn test_two() {
    let cdb = CDB::open("tests/test1.cdb").unwrap();
    assert_eq!(cdb.find(b"two").next().unwrap().unwrap(), b"Goodbye");
    assert_eq!(
        cdb.find(b"this key will be split across two reads")
            .next()
            .unwrap()
            .unwrap(),
        b"Got it."
    );
}

#[test]
fn test_empty_cdb() {
    let filename = "tests/empty.cdb";
    let mut cdb = CDBWriter::create(filename).unwrap();
    cdb.finish().unwrap();
    // make sure we can read what was just written.
    let cdb = CDB::open(filename);
    assert!(cdb.is_ok());

    let cdb = cdb.unwrap();

    // check that find works
    let iter = cdb.find(b"key");
    assert_eq!(iter.count(), 0);

    // check that get works
    let get_result = cdb.get(b"key");
    assert!(get_result.is_none());

    assert_eq!(cdb.iter().count(), 0);

    fs::remove_file(filename);
}