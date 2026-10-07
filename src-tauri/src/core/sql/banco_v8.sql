-- Nº de socio correlativo, como en el Libro de papel. Se asigna al crear (MAX+1) y nunca
-- se reutiliza: un socio retirado conserva su fila y su número, así que el siguiente socio
-- toma el número que sigue.
ALTER TABLE socio ADD COLUMN numero INTEGER;

-- Numera los socios ya existentes por orden de ingreso (desempate por id para ser estable).
UPDATE socio SET numero = (
    SELECT COUNT(*)
    FROM socio s2
    WHERE s2.fecha_ingreso < socio.fecha_ingreso
       OR (s2.fecha_ingreso = socio.fecha_ingreso AND s2.id <= socio.id)
);
