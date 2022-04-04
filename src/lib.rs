use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{Cursor, Write},
    path::Path,
};

pub mod camera;
pub mod geo_coords;
pub mod nwb;
pub mod world;

pub fn write_file<T: Serialize, P: AsRef<Path>>(
    value: &T,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;

    println!("Started writing file: {:?}", path.as_ref());

    let code = bincode::serialize(value)?;
    let result = zstd::encode_all(Cursor::new(code), 0)?;
    let mut file = File::create(&path)?;

    file.write_all(&result)?;

    println!("Finished writing file: {:?}", path.as_ref());

    Ok(())
}

pub fn read_file<T: DeserializeOwned, P: AsRef<Path>>(
    path: P,
) -> Result<T, Box<dyn std::error::Error>> {
    use std::fs::File;

    println!("Started reading file: {:?}", path.as_ref());

    let file = File::open(&path)?;

    let result = zstd::decode_all(file)?;
    let d = bincode::deserialize(&result)?;

    println!("Finished reading file: {:?}", path.as_ref());

    Ok(d)
}
