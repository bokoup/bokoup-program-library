import { Connection, PublicKey } from '@solana/web3.js';
import { Account as TokenAccount } from '@solana/spl-token';
import { AnchorWallet } from '@solana/wallet-adapter-react';
import { UI, Network, AdminSettings, PromoExtendeds, TokenMetadataProgram } from '@bokoup/bpl-token-metadata';

export type State = {
    network: Network,
    connection: Connection,
    wallet: AnchorWallet,
    walletConnected: boolean,
    program: TokenMetadataProgram,
    adminSettings: UI<AdminSettings>,
    promoExtendeds: PromoExtendeds,
    tokenAccounts: TokenAccounts,
    shopPromos: ShopPromos,
    shopTotal: ShopTotal,
    mintEvent: MintEvent,
    cart: Cart
};

export type Action =
    | { network: State['network'] }
    | { connection: State['connection'] }
    | { wallet: State['wallet']; walletConnected: State['walletConnected'] }
    | { program: State['program'] }
    | { adminSettings: State['adminSettings'] }
    | { promoExtendeds: State['promoExtendeds'] }
    | { tokenAccounts: State['tokenAccounts'] }
    | { shopPromos: State['shopPromos'] }
    | { shopTotal: State['shopTotal'] }
    | { mintEvent: State['mintEvent'] }
    | { cart: State['cart'] };

export type TokenAccounts = {
    [key: string]: TokenAccount | null;
};

export type ShopTotal = {
    quantity: number,
    subtotal: number,
    discount: number,
    total: number,
};

export type ShopPromo = {
    mint: PublicKey,
    discount: number,
    applied: number,
};

export type ShopPromos = {
    [key: string]: ShopPromo,
};

export type MintEvent = {
    slot: number,
    mintString: string,
};

export type Cart = {
    item_count: number,
    items_subtotal_price: number,
    original_total_price: number,
    total_discount: number,
    total_price: number,
    items: CartItem[]

};

export type CartItem = {
    product_id: string;
    variant_id: string;
    handle: string,
    quantity: number,
    final_line_price: number;
    final_price: number;
    total_discount: number;
};


