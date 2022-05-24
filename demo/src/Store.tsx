import React, { createContext, useReducer, ReactNode, FC, useEffect } from 'react';
import { Action, State, TokenAccounts, ShopPromos, MintEvent } from './types/types';
import { initialProducts, initialShopTotal, getShopTotal, getShopPromos } from './components/Shop';
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

export const PROMO1 = 'GeWRS2Det9da6K2xQw4Fd62Kv3qVQx1E3wsjAqk8DGs1';
export const PROMO2 = '9cppW5ugbEHygEicY8vWcgyCRNqkbdTiwjqtBDpH7913';

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
const network = process.env.REACT_APP_NETWORK_URL as Network
    || 'https://api.devnet.solana.com' as Network;
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
    shopPromos: {} as ShopPromos,
    shopTotal: initialShopTotal,
    mintEvent: {} as MintEvent
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

export async function getPromoExtended(state: State, dispatch: React.Dispatch<Action>, mintString: string) {
    const promoExtendeds = Object.assign({}, state.promoExtendeds);
    console.log(mintString);

    if (state.promoExtendeds[mintString]) {
        promoExtendeds[mintString] = await state.program.getPromoExtended(new PublicKey(mintString))
        dispatch({ promoExtendeds });
    }
}

export async function getPromoExtendeds(state: State, dispatch: React.Dispatch<Action>) {
    const promoExtendeds = await state.program.getPromoExtendeds([PROMO1, PROMO2].map(mintString => new PublicKey(mintString)));
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
        getShopPromos(state, dispatch);
        getShopTotal(state, dispatch);
        getTokenAccounts(state, dispatch);
    }, [state.walletConnected]);

    useEffect(() => {
        getShopPromos(state, dispatch);
    }, [state.products, state.tokenAccounts]);

    useEffect(() => {
        getShopTotal(state, dispatch);
    }, [state.shopPromos]);

    useEffect(() => {
        getShopPromos(state, dispatch);
    }, [state.shopTotal.subtotal, state.shopTotal.quantity, state.shopTotal.discount]);

    useEffect(() => {
        if (state.mintEvent.mintString && state.promoExtendeds[state.mintEvent.mintString]) {
            getPromoExtended(state, dispatch, state.mintEvent.mintString);
        }
    }, [state.mintEvent]);

    useEffect(() => {
        getAdminSettings(state, dispatch);
        getPromoExtendeds(state, dispatch);
        getTokenAccounts(state, dispatch);
        state.program.program.addEventListener("MintEvent", (event, slot) => {
            const mintEvent = { mintString: event.mint, slot } as MintEvent;
            dispatch({ mintEvent })
        })
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
