INSERT INTO listing_receipt (
    id,
    trade_state,
    bookkeeper,
    auction_house,
    seller,
    metadata,
    purchase_receipt,
    price,
    token_size,
    bump,
    trade_state_bump,
    created_at_on_chain,
    canceled_at_on_chain,
    slot,
    write_version
)
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
ON CONFLICT ON CONSTRAINT listing_receipt_pkey DO UPDATE 
    SET
        trade_state = EXCLUDED.trade_state,
        bookkeeper = EXCLUDED.bookkeeper,
        auction_house = EXCLUDED.auction_house,
        seller = EXCLUDED.seller,
        metadata = EXCLUDED.metadata,
        purchase_receipt = EXCLUDED.purchase_receipt,
        price = EXCLUDED.price,
        token_size = EXCLUDED.token_size,
        bump = EXCLUDED.bump,
        trade_state_bump = EXCLUDED.trade_state_bump,
        created_at_on_chain = EXCLUDED.created_at_on_chain,
        canceled_at_on_chain = EXCLUDED.canceled_at_on_chain,
        slot = EXCLUDED.slot,
        write_version = EXCLUDED.write_version,
        modified_at = NOW()
    WHERE
        EXCLUDED.slot > listing_receipt.slot
        OR (
            EXCLUDED.slot = listing_receipt.slot
            AND EXCLUDED.write_version > listing_receipt.write_version
        )
RETURNING created_at = modified_at