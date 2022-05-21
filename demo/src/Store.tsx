import React, { createContext, useReducer, ReactNode, FC, useEffect, useMemo } from 'react';
import { Action, State, TokenAccounts, } from './types/types';
import { initialProducts, initialShopTotal, getShopTotal } from './components/Shop';
import { Connection, ConfirmOptions, PublicKey } from '@solana/web3.js';
import {
    TokenMetadataProgram,
    AdminSettings,
    Network,
    PromoExtendeds,
    UI,
    Promo,
    PromoExtended,
} from '@bokoup/bpl-token-metadata';
import { AnchorProvider, BN } from '@project-serum/anchor';
import { Keypair, Transaction } from '@solana/web3.js';
import { AnchorWallet, useAnchorWallet } from '@solana/wallet-adapter-react';

const confirmOptions: ConfirmOptions = {
    skipPreflight: false,
    commitment: 'confirmed',
    preflightCommitment: 'processed',
    maxRetries: 10,
};

const dummyKeypair = Keypair.generate();
export const dummyWallet: AnchorWallet = {
    publicKey: dummyKeypair.publicKey,
    signTransaction: async (_transaction: Transaction): Promise<Transaction> => {
        return new Transaction();
    },
    signAllTransactions: async (_transactions: Transaction[]): Promise<Transaction[]> => {
        return [] as Transaction[];
    },
};
const network = process.env.REACT_APP_NETWORK_URL as Network;
const connection = new Connection(network, confirmOptions);
const provider = new AnchorProvider(connection, dummyWallet, confirmOptions);
const program = new TokenMetadataProgram(provider);
const initialAdminSettings = {
    publicKey: dummyKeypair.publicKey,
    platform: dummyKeypair.publicKey,
    createPromoLamports: new BN(0),
    burnPromoTokenLamports: new BN(0),
};

const initialState: State = {
    network,
    connection,
    wallet: dummyWallet,
    walletConnected: false,
    program,
    adminSettings: initialAdminSettings,
    promoExtendeds: {} as PromoExtendeds,
    tokenAccounts: {} as TokenAccounts,
    products: initialProducts,
    shopTotal: initialShopTotal
};

const Reducer = (state: State, action: Action): State => {
    return {
        ...state,
        ...action,
    };
};

export async function getAdminSettings(state: State, dispatch: React.Dispatch<Action>) {
    const [admin] = await state.program.findAdminAddress();
    const adminSettings = (await state.program.program.account.adminSettings.fetch(admin)) as UI<AdminSettings>;
    dispatch({ adminSettings });
}

export async function getPromoExtended(state: State, dispatch: React.Dispatch<Action>, promoExtended: PromoExtended) {
    const promoExtendeds = Object.assign({}, state.promoExtendeds);

    const promoExtendedUpdated = await state.program.program.account.promo
        .fetch(promoExtended.publicKey)
        .then((promoAccount) =>
            state.program.getPromoExtended({ publicKey: promoExtended.publicKey, ...promoAccount } as UI<Promo>)
        );

    promoExtendeds[promoExtendedUpdated.mintAccount.address.toString()] = promoExtendedUpdated;

    dispatch({ promoExtendeds });
}

export async function getPromoExtendeds(state: State, dispatch: React.Dispatch<Action>) {
    const promoExtendeds = await state.program.getPromos().then((res) => {
        return state.program.getPromoExtendeds(res);
    });
    dispatch({ promoExtendeds });
}

export async function getTokenAccount(state: State, dispatch: React.Dispatch<Action>, mint: PublicKey) {
    const tokenAccounts = Object.assign({}, state.tokenAccounts);

    const tokenAccount = await state.program
        .findAssociatedTokenAccountAddress(mint, state.wallet.publicKey)
        .then(([address]) => state.program.getTokenAccount(address))
        .catch(() => null);

    tokenAccounts[mint.toString()] = tokenAccount;
    dispatch({ tokenAccounts });
}

export async function getTokenAccounts(state: State, dispatch: React.Dispatch<Action>) {
    const results = await Promise.all(
        Object.keys(state.promoExtendeds).map((mintString) =>
            state.program
                .findAssociatedTokenAccountAddress(new PublicKey(mintString), state.wallet.publicKey)
                .then(([address]) => state.program.getTokenAccount(address))
                .catch(() => null)
        )
    );

    const tokenAccounts: TokenAccounts = Object.keys(state.promoExtendeds).reduce(
        (tokenAccounts, mintString, i) => ((tokenAccounts[mintString] = results[i]), tokenAccounts),
        {} as TokenAccounts
    );
    dispatch({ tokenAccounts });
}

export function getDemoKeypair(secretKeyString: string): Keypair {
    return Keypair.fromSecretKey(new Uint8Array(JSON.parse(secretKeyString)));
}

const Store: FC<{ children: ReactNode }> = ({ children }) => {
    console.log("renderStore");
    const [state, dispatch] = useReducer(Reducer, initialState);

    let wallet = useAnchorWallet();
    useEffect(() => {
        dispatch({ wallet: wallet ? wallet : dummyWallet, walletConnected: wallet ? true : false });
    }, [wallet]);

    useEffect(() => {
        const connection = new Connection(state.network, confirmOptions);
        dispatch({ connection });
    }, [state.network]);


    useEffect(() => {
        let provider = new AnchorProvider(state.connection, dummyWallet, confirmOptions);

        if (state.walletConnected) {
            provider = new AnchorProvider(state.connection, state.wallet, confirmOptions);
        }
        const program = new TokenMetadataProgram(provider);
        dispatch({ program });

        getTokenAccounts(state, dispatch);
    }, [state.walletConnected]);

    useEffect(() => {
        getShopTotal(state, dispatch);
        getTokenAccounts(state, dispatch);
    }, [state.walletConnected]);

    useEffect(() => {
        // getTokenAccounts(state, dispatch);
        getShopTotal(state, dispatch);
    }, [state.products, state.tokenAccounts]);

    useMemo(() => {
        getAdminSettings(state, dispatch);
        getPromoExtendeds(state, dispatch);
        getTokenAccounts(state, dispatch);
        getShopTotal(state, dispatch);
    }, []);

    return <Context.Provider value={{ state, dispatch }}>{children}</Context.Provider>;
};

const Context = createContext<{
    state: State;
    dispatch: React.Dispatch<Action>;
}>({
    state: initialState,
    dispatch: () => null,
});
export { Context, Store };
