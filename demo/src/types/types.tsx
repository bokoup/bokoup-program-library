import { Connection, PublicKey } from '@solana/web3.js';
import { Account as TokenAccount } from '@solana/spl-token';
import { AnchorWallet } from '@solana/wallet-adapter-react';
import { UI, Network, AdminSettings, PromoExtendeds, TokenMetadataProgram } from '@bokoup/bpl-token-metadata';

export type State = {
    network: Network;
    connection: Connection;
    wallet: AnchorWallet;
    walletConnected: boolean;
    program: TokenMetadataProgram;
    adminSettings: UI<AdminSettings>;
    promoExtendeds: PromoExtendeds;
    tokenAccounts: TokenAccounts;
    products: Products;
    shopPromos: ShopPromos;
    shopTotal: ShopTotal;
};

export type Action =
    | { network: State['network'] }
    | { connection: State['connection'] }
    | { wallet: State['wallet']; walletConnected: State['walletConnected'] }
    | { program: State['program'] }
    | { adminSettings: State['adminSettings'] }
    | { promoExtendeds: State['promoExtendeds'] }
    | { tokenAccounts: State['tokenAccounts'] }
    | { products: State['products'] }
    | { shopPromos: State['shopPromos'] }
    | { shopTotal: State['shopTotal'] };

export type TokenAccounts = {
    [key: string]: TokenAccount | null;
};

export type Products = {
    [key: string]: Product;
};

export type Product = {
    name: string,
    description: string,
    src: '*.png',
    price: number,
    quantity: number,
    total: number,
};

export type ShopTotal = {
    quantity: number,
    subtotal: number,
    discount: number;
    total: number,
};

export type ShopPromo = {
    mint: PublicKey,
    discount: number,
    applied: number
}

export type ShopPromos = {
    [key: string]: ShopPromo;
};
