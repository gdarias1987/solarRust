# Content provenance

`channels.json` is a field-preserving extraction of Solar Go `internal/sdo/filters.go`
from the local working tree on 2026-11-10 (not a scientific revalidation).
Source SHA-256: `fdbb55e384e2e516e3f7dbad46dd71d47cd89ed98da9d8827c0ef6ec80e4786c`.
Explicit URLs, names, descriptions, ions, regions and Kelvin ranges are preserved.
Temperatures displayed in Celsius/Fahrenheit are arithmetic conversions.
NASA product availability may change.

## Bundled image previews

The JPEG files in `content/previews/` are static preview assets used by the interface; they are
not the live captures downloaded by the application. They were copied from the Solar Go working
tree and originate from Solar Dynamics Observatory (SDO) imagery. The application credits the
imagery as “NASA / SDO” in its About screen. These assets are not covered by this repository's
Apache-2.0 code license; consult the [SDO image-use and copyright guidance](https://sdo.gsfc.nasa.gov/gallery/copyright/)
for their applicable terms. The live captures are fetched from the SDO URLs listed in
`channels.json` and stored locally by the application.
