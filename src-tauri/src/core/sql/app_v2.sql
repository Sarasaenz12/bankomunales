-- RF-04: el nombre de un Bankomunal es único entre todos los del catálogo,
-- sin distinción de mayúsculas/minúsculas. Este índice es la fuente de verdad
-- de la unicidad y cierra la condición de carrera del chequeo previo en aplicación.
CREATE UNIQUE INDEX IF NOT EXISTS idx_catalogo_bankomunal_nombre
    ON catalogo_bankomunal (nombre COLLATE NOCASE);
