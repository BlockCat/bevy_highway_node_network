# Dutch roads

![map](docs/map.png)

## Install:
1. Retrieve latest data from https://www.rijkswaterstaat.nl/apps/geoservices/geodata/dmc/nwb-wegen/geogegevens/shapefile/Nederland_totaal/
2. Extract contents to /data folder.
3. Create a database using the tools/dbf_to_sql tool.


## Changes:

Changed old neighbourhood using hashmap to new neighbourhood using radius:

|Ver|Old|Old|New|New|
|---|---|---|---|---|
| N | 4 | 30| 4 | 30|
|ms |696| 4453| 452 |3682|

Phase 1 generation:
```
time:   [20.726 s 22.190 s 23.603 s]
```
