INSERT INTO promo_group (
    id,
    owner,
    seed,
    nonce,
    members,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT ON CONSTRAINT promo_group_pkey DO UPDATE 
    SET
        owner = EXCLUDED.owner,
        seed = EXCLUDED.seed,
        nonce = EXCLUDED.nonce,
        members = EXCLUDED.members,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > promo_group.slot
        OR (
            EXCLUDED.slot = promo_group.slot
            AND EXCLUDED.write_version > promo_group.write_version
        )
RETURNING created_at = modified_at