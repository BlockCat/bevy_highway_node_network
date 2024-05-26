# Dutch roads

![map](docs/map.png)

## Install:

1. Retrieve latest data from https://www.rijkswaterstaat.nl/apps/geoservices/geodata/dmc/nwb-wegen/geogegevens/shapefile/Nederland_totaal/
2. Extract contents to /data folder.
3. Create a database using the tools/dbf_to_sql tool. (`cargo.exe run --package dbf_to_sql --bin dbf_to_sql --release`)

## Structure:

### Visualization

- main package: Uses bevy for visualization (using weird stuff)
- bevy_shapefile: Load spatial data for visualization

## Changes:

Changed old neighbourhood using hashmap to new neighbourhood using radius:

| Ver | Old | Old  | New | New  |
| --- | --- | ---- | --- | ---- |
| N   | 4   | 30   | 4   | 30   |
| ms  | 696 | 4453 | 452 | 3682 |

Phase 1 generation:

```
time:   [20.726 s 22.190 s 23.603 s]
```

## Types

https://docs.ndw.nu/handleidingen/nwb/nwb-basisstructuur/overige-attributen/

- VWG (Ventweg)
- PAR (Parallelweg - niet ventweg)
- MRB(Minirotondebaan)
- NRB (Normale rotondebaan - niet minirotondebaan)
- OPR (Toerit - synoniem: oprit)
- AFR (Afrit)
- PST (Puntstuk = snijpunt verharding)
- VBD (Verbindingsweg direct)
- VBI (Verbindingsweg indirect)
- VBS (Verbindingsweg semi-direct)
- VBR (Verbindingsweg rangeerbaan)
- VBK (Verbindingsweg kortsluitend)
- VBW(Verbindingsweg - overig)
- DST (Doorsteek)
- PKP (Verzorgingsbaan van/naar parkeerplaats)
- PKB (verzorgingsbaan van/naar parkeerplaats bij benzinestation)
- BST (Verzorgingsbaan van /naar benzinestation)
- YYY (Overige baan)
- BU(Busbaan)
- FP (Fietspad)
- HR (Hoofdrijbaan)
- TN(Tussenbaan)
- VP(Voetpad)
- OVB (OV-baan)
- CADO (Calamiteiten doorgang)
- TRB (Turborotondebaan)
- RP(Ruiterpad)
- VV (Vliegverkeer)
- PP (Parkeerplaats)
- PC(Parkeerplaats tbv carpool)
- PR(Parkeerplaats P+R)
- VD (Veerdienst)
- (Geen)
