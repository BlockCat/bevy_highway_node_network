INSERT INTO wegvakken
(
    id,
    wegvak_id,

    junction_id_begin,
    junction_id_end,

    rij_richting,

    straat_naam,
    beheerder,
    
    huisnummer_structuur_links,
    huisnummer_structuur_rechts,

    eerste_huisnummer_links,
    eerste_huisnummer_rechts,

    laatste_huisnummer_links,
    laatste_huisnummer_rechts,

    begin_afstand,
    eind_afstand,

    begin_km,
    eind_km
) VALUES (
    ?, 
    ?,

    ?,
    ?,

    ?,

    ?,
    ?,
    
    ?,
    ?,
    
    ?,
    ?,

    ?,
    ?,

    ?,
    ?,

    ?,
    ?
);