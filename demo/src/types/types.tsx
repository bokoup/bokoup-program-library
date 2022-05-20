import { Connection } from '@solana/web3.js';
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
};

export type Action =
    | { network: State['network'] }
    | { connection: State['connection'] }
    | { wallet: State['wallet']; walletConnected: State['walletConnected'] }
    | { program: State['program'] }
    | { adminSettings: State['adminSettings'] }
    | { promoExtendeds: State['promoExtendeds'] }
    | { tokenAccounts: State['tokenAccounts'] }
    | { products: State['products'] };

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

