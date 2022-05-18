import React, { createContext, useReducer, ReactNode, FC, useEffect, useMemo } from "react";
import { Action, State, TokenAccounts } from './types/types'
import { Connection, ConfirmOptions, PublicKey } from '@solana/web3.js';
import { TokenMetadataProgram, AdminSettings, Network, PromoExtendeds, UI } from '@bokoup/bpl-token-metadata';
import { AnchorProvider, BN } from '@project-serum/anchor';
import { Keypair, Transaction } from '@solana/web3.js';
import { AnchorWallet, useAnchorWallet } from '@solana/wallet-adapter-react';

const confirmOptions: ConfirmOptions = {
    skipPreflight: false,
    commitment: "confirmed",
    preflightCommitment: "processed",
    maxRetries: 10
};

const dummyKeypair = Keypair.generate();
export const dummyWallet: AnchorWallet = {
    publicKey: dummyKeypair.publicKey,
    signTransaction: async (transaction: Transaction): Promise<Transaction> => {
        return new Transaction();
    },
    signAllTransactions: async (transactions: Transaction[]): Promise<Transaction[]> => {
        return [] as Transaction[];
    }
}
const network = process.env.REACT_APP_NETWORK_URL as Network;
const connection = new Connection(network, confirmOptions);
const provider = new AnchorProvider(connection, dummyWallet, confirmOptions);
const program = new TokenMetadataProgram(provider);
const initialAdminSettings = { publicKey: dummyKeypair.publicKey, platform: dummyKeypair.publicKey, createPromoLamports: new BN(0), burnPromoTokenLamports: new BN(0) }

const initialState: State = {
    network,
    connection,
    wallet: dummyWallet,
    walletConnected: false,
    program,
    adminSettings: initialAdminSettings,
    promoExtendeds: {} as PromoExtendeds,
    tokenAccounts: {} as TokenAccounts

}

const Reducer = (state: State, action: Action): State => {
    return {
        ...state,
        ...action
    };
};

export async function getAdminSettings(state: State, dispatch: React.Dispatch<Action>) {
    const [admin] = await state.program.findAdminAddress();
    const adminSettings = await state.program.program.account.adminSettings.fetch(admin) as UI<AdminSettings>;
    dispatch({ adminSettings })
}

export async function getPromoExtendeds(state: State, dispatch: React.Dispatch<Action>) {
    const promoExtendeds = await state.program.getPromos().then(res => {
        return state.program.getPromoExtendeds(res)
    });
    dispatch({ promoExtendeds })
}

export async function getTokenAccounts(state: State, dispatch: React.Dispatch<Action>) {
    const results = await Promise.all(Object.keys(state.promoExtendeds).map(mintString => state.program.findAssociatedTokenAccountAddress(new PublicKey(mintString), state.wallet.publicKey)
        .then(([address]) => { return state.program.getTokenAccount(address) })
        .catch(() => { return null })
    ));
    console.log(results);

    const tokenAccounts: TokenAccounts = Object.keys(state.promoExtendeds).reduce(
        (tokenAccounts, mintString, i) => (
            (tokenAccounts[mintString] = results[i]),
            tokenAccounts
        ),
        {} as TokenAccounts,
    );
    dispatch({ tokenAccounts })
}

const Store: FC<{ children: ReactNode }> = ({ children }) => {
    const [state, dispatch] = useReducer(Reducer, initialState);

    const wallet = useAnchorWallet();
    useEffect(() => {
        dispatch({ wallet: wallet ? wallet : dummyWallet, walletConnected: wallet ? true : false })
    }, [wallet]);

    useMemo(() => {
        getTokenAccounts(state, dispatch);
    }, [state.wallet]);

    useEffect(() => {
        const connection = new Connection(state.network, confirmOptions);
        dispatch({ connection })
    },
        [state.network]);

    useEffect(() => {
        let provider = new AnchorProvider(
            state.connection, dummyWallet, confirmOptions,
        );

        if (state.walletConnected) {
            provider = new AnchorProvider(
                state.connection, state.wallet, confirmOptions,
            )
        }
        const program = new TokenMetadataProgram(provider);
        dispatch({ program })
    }, [state.wallet]);

    useMemo(() => {
        getAdminSettings(state, dispatch);
        getPromoExtendeds(state, dispatch);
        getTokenAccounts(state, dispatch);
    }, []);

    return (
        <Context.Provider value={{ state, dispatch }}>
            {children}
        </Context.Provider>
    )
};

const Context = createContext<{
    state: State,
    dispatch: React.Dispatch<Action>
}>({
    state: initialState,
    dispatch: () => null
});
export { Context, Store };