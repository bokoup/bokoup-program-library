DELETE FROM creator WHERE
    metadata = $1 AND
    NOT address = ANY($2::text[])
RETURNING address