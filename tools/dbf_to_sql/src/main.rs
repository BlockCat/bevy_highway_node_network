use std::path::Path;

use rusqlite::{params, Connection, Transaction};
use shapefile::dbase::{FieldValue, Record};

const CREATE_TABLE_SQL: &'static str = include_str!("create_table.sql");
const INSERT_SQL: &'static str = include_str!("insert.sql");

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
            stmt.execute(params![
                id,
                get_usize(&record, "WVK_ID"),
                get_usize(&record, "JTE_ID_BEG"),
                get_usize(&record, "JTE_ID_END"),
                get_text(&record, "RIJRICHTNG"),
                get_text(&record, "STT_NAAM"),
                get_text(&record, "WEGBEHNAAM"),
                get_text(&record, "WEGTYPE"),
                get_text(&record, "WGTYPE_OMS"),
                get_text(&record, "HNRSTRLNKS"),
                get_text(&record, "HNRSTRRHTS"),
                get_usize(&record, "E_HNR_LNKS"),
                get_usize(&record, "E_HNR_RHTS"),
                get_usize(&record, "L_HNR_LNKS"),
                get_usize(&record, "L_HNR_RHTS"),
                get_float(&record, "BEGAFSTAND"),
                get_float(&record, "ENDAFSTAND"),
                get_float(&record, "BEGINKM"),
                get_float(&record, "EINDKM")
            ])
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
        return x.clone();
    }
    unreachable!();
}
