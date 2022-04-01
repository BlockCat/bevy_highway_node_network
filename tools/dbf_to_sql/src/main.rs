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

    execute_transaction("data/01-03-2022/Wegvakken/Wegvakken.shp", &transaction);

    println!("Start committing");

    transaction.commit().expect("Could not commit");

    println!("Finish committing");
}

fn execute_transaction<P: AsRef<Path>>(path: P, tx: &Transaction) {
    let mut reader = shapefile::Reader::from_path(path).expect("Could not find path");

    reader
        .iter_shapes_and_records()
        .enumerate()
        .for_each(|(id, result)| {
            let (_, record) = result.unwrap();

            tx.execute(
                INSERT_SQL,
                params![
                    id,
                    get_usize(&record, "WVK_ID"),
                    get_usize(&record, "JTE_ID_BEG"),
                    get_usize(&record, "JTE_ID_END"),
                    get_char(&record, "RIJRICHTNG"),
                    get_char(&record, "STT_NAAM"),
                    get_char(&record, "WEGBEHNAAM"),
                    get_char(&record, "HNRSTRLNKS"),
                    get_char(&record, "HNRSTRRHTS"),
                    get_usize(&record, "E_HNR_LNKS"),
                    get_usize(&record, "E_HNR_RHTS"),
                    get_usize(&record, "L_HNR_LNKS"),
                    get_usize(&record, "L_HNR_RHTS"),
                    get_float(&record, "BEGAFSTAND"),
                    get_float(&record, "ENDAFSTAND"),
                    get_float(&record, "BEGINKM"),
                    get_float(&record, "EINDKM")
                ],
            )
            .expect("Could not insert");
        });
}

fn get_char(record: &Record, name: &str) -> Option<String> {
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
