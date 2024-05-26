CREATE TABLE IF NOT EXISTS wegvakken
(
    id INTEGER UNIQUE,
    wegvak_id INTEGER UNIQUE, --WVK_ID
    
    junction_id_begin INTEGER, --JTE_ID_BEG
    junction_id_end INTEGER, --JTE_ID_END

    rij_richting CHARACTER(1), --RIJRICHTNG

    straat_naam TEXT, --STT_NAAM
    beheerder TEXT, --WEGBEHNAAM

    weg_type TEXT, -- WEGTYPE

    huisnummer_structuur_links CHARACTER(1), --HNRSTRLNKS
    huisnummer_structuur_rechts CHARACTER(1), --HNRSTRRHTS

    eerste_huisnummer_links INTEGER, --E_HNR_LNKS
    eerste_huisnummer_rechts INTEGER, --E_HNR_RHTS

    laatste_huisnummer_links INTEGER, --L_HNR_LNKS
    laatste_huisnummer_rechts INTEGER, --L_HNR_RHTS

    begin_afstand REAL, --BEGAFSTAND
    eind_afstand REAL, --ENDAFSTAND

    begin_km REAL, --BEGINKM
    eind_km REAL --EINDKM

);