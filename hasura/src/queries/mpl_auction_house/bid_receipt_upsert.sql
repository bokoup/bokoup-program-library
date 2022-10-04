INSERT INTO bid_receipt (
    id,
    trade_state,
    bookkeeper,
    auction_house,
    buyer,
    metadata,
    token_account,
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
    VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
ON CONFLICT ON CONSTRAINT bid_receipt_pkey DO UPDATE 
    SET
        trade_state = EXCLUDED.trade_state,
        bookkeeper = EXCLUDED.bookkeeper,
        auction_house = EXCLUDED.auction_house,
        buyer = EXCLUDED.buyer,
        metadata = EXCLUDED.metadata,
        token_account = EXCLUDED.token_account,
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
        EXCLUDED.slot > bid_receipt.slot
        OR (
            EXCLUDED.slot = bid_receipt.slot
            AND EXCLUDED.write_version > bid_receipt.write_version
        )
RETURNING created_at = modified_at