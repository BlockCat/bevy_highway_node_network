use std::path::Path;

use rusqlite::{named_params, Connection, Transaction};
use shapefile::dbase::{FieldValue, Record};

pub const SKIP_TYPES: [&'static str; 7] = ["FP", "BU", "VP", "OVB", "CADO", "RP", "VV"];

const CREATE_TABLE_SQL: &str = include_str!("create_table.sql");
const INSERT_SQL: &str = include_str!("insert.sql");

fn main() {
    let mut connection = Connection::open("database.db").expect("Could not create database");

    connection
        .execute("DROP TABLE IF EXISTS wegvakken", [])
        .expect("Could not delete table");

    connection
        .execute(CREATE_TABLE_SQL, [])
        .expect("Could not create tables");

    let transaction = connection
        .transaction()
        .expect("Could not start transaction");

    execute_transaction("data/01-12-2022/Wegvakken/Wegvakken.shp", &transaction);

    println!("Start committing");

    transaction.commit().expect("Could not commit");

    println!("Finish committing");
}

fn execute_transaction<P: AsRef<Path>>(path: P, tx: &Transaction) {
    println!("Start reading");
    let collection = shapefile::read(path).expect("Could not read path");
    println!("Finished reading");

    let mut stmt = tx.prepare(INSERT_SQL).unwrap();

    collection
        .into_iter()
        .enumerate()
        .for_each(|(id, (_, record))| {
            stmt.execute(named_params! {
                         ":id": id,
            ":wegvak_id": get_usize(&record, "WVK_ID"),
            ":junction_id_begin": get_usize(&record, "JTE_ID_BEG"),
            ":junction_id_end": get_usize(&record, "JTE_ID_END"),
            ":rij_richting": get_text(&record, "RIJRICHTNG"),
            ":straat_naam": get_text(&record, "STT_NAAM"),
            ":beheerder": get_text(&record, "WEGBEHNAAM"),
            ":weg_type": get_text(&record, "BST_CODE"),
            ":huisnummer_structuur_links": get_text(&record, "HNRSTRLNKS"),
            ":huisnummer_structuur_rechts": get_text(&record, "HNRSTRRHTS"),
            ":eerste_huisnummer_links": get_usize(&record, "E_HNR_LNKS"),
            ":eerste_huisnummer_rechts": get_usize(&record, "E_HNR_RHTS"),
            ":laatste_huisnummer_links": get_usize(&record, "L_HNR_LNKS"),
            ":laatste_huisnummer_rechts": get_usize(&record, "L_HNR_RHTS"),
            ":begin_afstand": get_float(&record, "BEGAFSTAND"),
            ":eind_afstand": get_float(&record, "ENDAFSTAND"),
            ":begin_km": get_float(&record, "BEGINKM"),
            ":eind_km": get_float(&record, "EINDKM"),
                    })
            .expect("Could not insert");
        });
}

fn get_text(record: &Record, name: &str) -> Option<String> {
    let value = record.get(name).unwrap();

    if let FieldValue::Character(x) = value {
        return x.clone();
    }
    unreachable!()
}

fn get_usize(record: &Record, name: &str) -> Option<usize> {
    let value = record.get(name).unwrap();

    if let FieldValue::Numeric(x) = value {
        return x.map(|x| x as usize);
    }
    unreachable!();
}

fn get_float(record: &Record, name: &str) -> Option<f64> {
    let value = record.get(name).unwrap();

    if let FieldValue::Numeric(x) = value {
        return *x;
    }
    unreachable!();
}
