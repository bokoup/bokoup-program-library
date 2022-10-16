INSERT INTO creator (
    metadata,
    address,
    verified,
    share,
    slot,
    write_version
)
SELECT * FROM UNNEST($1::text[], $2::text[], $3::bool[], $4::int[], $5::bigint[], $6::bigint[])
ON CONFLICT ON CONSTRAINT creator_pkey DO UPDATE 
    SET
        metadata = EXCLUDED.metadata,
        address = EXCLUDED.address,
        verified = EXCLUDED.verified,
        share = EXCLUDED.share,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > creator.slot
        OR (
            EXCLUDED.slot = creator.slot
            AND EXCLUDED.write_version > creator.write_version
        )
RETURNING created_at = modified_at