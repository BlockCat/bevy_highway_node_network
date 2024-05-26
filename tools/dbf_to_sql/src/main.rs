use rayon::prelude::*;
use rusqlite::{Connection, ToSql, Transaction};
use shapefile::dbase::{FieldValue, Record};
use std::{collections::HashMap, path::Path, thread};

const CREATE_TABLE_SQL: &'static str = include_str!("create_table.sql");
const INSERT_SQL: &'static str = include_str!("insert_wegvak.sql");

fn main() {
    let mut connection = Connection::open("data/database.db").expect("Could not create database");

    prepare_database(&connection);

    let transaction = connection
        .transaction()
        .expect("Could not start transaction");

    let j1 = thread::spawn(|| read_wegvakken("data/01-05-2024/Wegvakken/Wegvakken.shp"));
    let j2 = thread::spawn(|| read_snelheidslimieten("data/01-05-2024/Snelheden.shp"));
    let snelheidslimieten = j2.join().unwrap();
    let mut wegvakken = j1.join().unwrap();

    println!("Start enhancing wegvakken with snelheidslimieten");

    enhance_wegvakken_with_snelheidslimieten(&mut wegvakken, snelheidslimieten);

    println!("Start executing transaction");

    execute_transaction(wegvakken, &transaction);

    println!("Start committing");

    transaction.commit().expect("Could not commit");

    println!("Finish committing");
}

fn prepare_database(connection: &Connection) {
    connection
        .execute("DROP TABLE IF EXISTS wegvakken", [])
        .expect("Could not delete table");

    connection
        .execute(CREATE_TABLE_SQL, [])
        .expect("Could not create tables");
}

fn read_shapefile<P: AsRef<Path>>(path: P) -> Vec<(shapefile::Shape, Record)> {
    println!("Start reading: {}", path.as_ref().display());
    let collection = shapefile::read(&path).expect("Could not read path");
    println!("Finished reading: {}", path.as_ref().display());

    collection
}

fn read_wegvakken<P: AsRef<Path>>(path: P) -> Vec<WegvakEntry> {
    let shapefile = read_shapefile(path);

    shapefile
        .into_par_iter()
        .enumerate()
        .map(|(id, (_, record))| WegvakEntry {
            id,
            wegvak_id: get_usize(&record, "WVK_ID").unwrap(),
            junction_id_begin: get_usize(&record, "JTE_ID_BEG").unwrap(),
            junction_id_end: get_usize(&record, "JTE_ID_END").unwrap(),
            rij_richting: get_text(&record, "RIJRICHTNG").unwrap(),
            straat_naam: get_text(&record, "STT_NAAM").unwrap(),
            beheerder: get_text(&record, "WEGBEHNAAM").unwrap(),
            // weg_type_category: get_text(&record, "BRT_CODE"),
            weg_type_subcategory: get_text(&record, "BST_CODE"),
            huisnummer_structuur_links: get_text(&record, "HNRSTRLNKS"),
            huisnummer_structuur_rechts: get_text(&record, "HNRSTRRHTS"),
            eerste_huisnummer_links: get_usize(&record, "E_HNR_LNKS"),
            eerste_huisnummer_rechts: get_usize(&record, "E_HNR_RHTS"),
            laatste_huisnummer_links: get_usize(&record, "L_HNR_LNKS"),
            laatste_huisnummer_rechts: get_usize(&record, "L_HNR_RHTS"),
            begin_afstand: get_float(&record, "BEGAFSTAND"),
            eind_afstand: get_float(&record, "ENDAFSTAND"),
            begin_km: get_float(&record, "BEGINKM"),
            eind_km: get_float(&record, "EINDKM"),
            snelheidslimiet: None,
        })
        .collect()
}

fn read_snelheidslimieten<P: AsRef<Path>>(path: P) -> HashMap<usize, usize> {
    let shapefile = read_shapefile(path);

    shapefile
        .into_par_iter()
        .map(|(_, record)| {
            let wegvak_id = get_usize(&record, "WVK_ID").unwrap();
            let snelheidslimiet = get_text(&record, "MAXSHD").unwrap().parse().unwrap_or(25);

            (wegvak_id, snelheidslimiet)
        })
        .collect()
}

fn enhance_wegvakken_with_snelheidslimieten(
    wegvakken: &mut Vec<WegvakEntry>,
    snelheidslimieten: HashMap<usize, usize>,
) {
    for entry in wegvakken.iter_mut() {
        if let Some(snelheidslimiet) = snelheidslimieten.get(&entry.wegvak_id) {
            entry.snelheidslimiet = Some(*snelheidslimiet);
        }
    }
}

fn execute_transaction(wegvakken: Vec<WegvakEntry>, tx: &Transaction) {
    let mut stmt = tx.prepare(INSERT_SQL).unwrap();

    wegvakken.into_iter().for_each(|wegvak_entry| {
        stmt.execute(&wegvak_entry.bind())
            .expect("Could not insert");
    });
}

fn get_text(record: &Record, name: &str) -> Option<String> {
    let value = record
        .get(name)
        .expect(format!("Could not find field: {}, records: {:?}", name, record).as_str());

    if let FieldValue::Character(x) = value {
        return x.clone();
    }
    unreachable!("Could not get text: {:?}", value);
}

fn get_usize(record: &Record, name: &str) -> Option<usize> {
    let value = record.get(name).unwrap();

    if let FieldValue::Numeric(x) = value {
        return x.map(|x| x as usize);
    }
    unreachable!("Could not get usize: {:?}", value);
}

fn get_float(record: &Record, name: &str) -> Option<f64> {
    let value = record.get(name).unwrap();

    if let FieldValue::Numeric(x) = value {
        return *x;
    }
    unreachable!("Could not get float: {:?}", value);
}

struct WegvakEntry {
    id: usize,
    wegvak_id: usize,
    junction_id_begin: usize,
    junction_id_end: usize,
    rij_richting: String,
    straat_naam: String,
    beheerder: String,
    // weg_type_category: Option<String>,
    weg_type_subcategory: Option<String>,
    huisnummer_structuur_links: Option<String>,
    huisnummer_structuur_rechts: Option<String>,
    eerste_huisnummer_links: Option<usize>,
    eerste_huisnummer_rechts: Option<usize>,
    laatste_huisnummer_links: Option<usize>,
    laatste_huisnummer_rechts: Option<usize>,
    begin_afstand: Option<f64>,
    eind_afstand: Option<f64>,
    begin_km: Option<f64>,
    eind_km: Option<f64>,
    snelheidslimiet: Option<usize>,
}

impl WegvakEntry {
    pub fn bind(&self) -> [(&str, &dyn ToSql); 19] {
        [
            (":id", &self.id),
            (":wegvak_id", &self.wegvak_id),
            (":junction_id_begin", &self.junction_id_begin),
            (":junction_id_end", &self.junction_id_end),
            (":rij_richting", &self.rij_richting),
            (":straat_naam", &self.straat_naam),
            (":beheerder", &self.beheerder),
            // (":weg_type_category", &self.weg_type_category),
            (":weg_type_subcategory", &self.weg_type_subcategory),
            (
                ":huisnummer_structuur_links",
                &self.huisnummer_structuur_links,
            ),
            (
                ":huisnummer_structuur_rechts",
                &self.huisnummer_structuur_rechts,
            ),
            (":eerste_huisnummer_links", &self.eerste_huisnummer_links),
            (":eerste_huisnummer_rechts", &self.eerste_huisnummer_rechts),
            (":laatste_huisnummer_links", &self.laatste_huisnummer_links),
            (
                ":laatste_huisnummer_rechts",
                &self.laatste_huisnummer_rechts,
            ),
            (":begin_afstand", &self.begin_afstand),
            (":eind_afstand", &self.eind_afstand),
            (":begin_km", &self.begin_km),
            (":eind_km", &self.eind_km),
            (":snelheidslimiet", &self.snelheidslimiet),
        ]
    }
}
