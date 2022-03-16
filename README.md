# Ziele / Fragen

- Test coverage verbessern
- Todos abarbeiten
- Bessere Error Messages
- Paket Format zur besseren Beschreibung und zum building erstellen
- inkonsistente IO-Operationen überarbeiten
- Datei Rechte

# Bisher

- etwas zu lange nach einer DB crate gesucht:
    - sled -> nur für länger anhaltenden Prozesse
    - sqlite -> war mir etwas zu komplex
    - nun sehr einfaches serializing/deserializing ohne partial loading oder cache (cache bei kurz lebigen prozessen eh eher egal)
    - komplette Daten werden in Memory geladen -> Store mit sehr vielen Paketen könnte Probleme bringen