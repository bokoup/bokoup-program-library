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
};

export type Action =
    | { network: State['network'] }
    | { connection: State['connection'] }
    | { wallet: State['wallet']; walletConnected: State['walletConnected'] }
    | { program: State['program'] }
    | { adminSettings: State['adminSettings'] }
    | { promoExtendeds: State['promoExtendeds'] }
    | { tokenAccounts: State['tokenAccounts'] };

export type TokenAccounts = {
    [key: string]: TokenAccount | null;
};
