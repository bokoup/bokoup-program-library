import { Wallet } from '@project-serum/anchor';

import {
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmRawTransaction,
  SystemProgram,
  Transaction,
} from '@solana/web3.js';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  getMinimumBalanceForRentExemptMint,
  createInitializeMintInstruction,
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';

/**
 * Create and initialize a new mint
 *
 * @param connection      Connection to use
 * @param payer           Payer of the transaction and initialization fees
 * @param mintAuthority   Account or multisig that will control minting
 * @param freezeAuthority Optional account or multisig that can freeze token accounts
 * @param decimals        Location of the decimal place
 * @param keypair         Optional keypair, defaulting to a new random one
 * @param confirmOptions  Options for confirming the transaction
 * @param programId       SPL Token program account
 *
 * @return Address of the new mint
 */

export async function createMint(
  connection: Connection,
  payer: Wallet,
  mintAuthority: PublicKey,
  freezeAuthority: PublicKey | null,
  decimals: number,
  keypair = Keypair.generate(),
  confirmOptions?: ConfirmOptions,
  programId = TOKEN_PROGRAM_ID,
): Promise<PublicKey> {
  const lamports = await getMinimumBalanceForRentExemptMint(connection);
  const keyPairWallet = new Wallet(keypair);

  const transaction = new Transaction({ feePayer: payer.publicKey }).add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: keypair.publicKey,
      space: MINT_SIZE,
      lamports,
      programId,
    }),
    createInitializeMintInstruction(
      keypair.publicKey,
      decimals,
      mintAuthority,
      freezeAuthority,
      programId,
    ),
  );
  transaction.recentBlockhash = await (await connection.getLatestBlockhash()).blockhash;
  await Promise.all([
    payer.signTransaction(transaction),
    keyPairWallet.signTransaction(transaction),
  ]);

  await sendAndConfirmRawTransaction(connection, transaction.serialize(), confirmOptions);

  return keypair.publicKey;
}

/**
 * Create and initialize a new associated token account
 *
 * @param connection               Connection to use
 * @param payer                    Payer of the transaction and initialization fees
 * @param mint                     Mint for the account
 * @param owner                    Owner of the new account
 * @param confirmOptions           Options for confirming the transaction
 * @param programId                SPL Token program account
 * @param associatedTokenProgramId SPL Associated Token program account
 *
 * @return Address of the new associated token account
 */
export async function createAssociatedTokenAccount(
  connection: Connection,
  payer: Wallet,
  mint: PublicKey,
  owner: PublicKey,
  confirmOptions?: ConfirmOptions,
  programId = TOKEN_PROGRAM_ID,
  associatedTokenProgramId = ASSOCIATED_TOKEN_PROGRAM_ID,
): Promise<PublicKey> {
  const associatedToken = await getAssociatedTokenAddress(
    mint,
    owner,
    false,
    programId,
    associatedTokenProgramId,
  );

  const transaction = new Transaction({ feePayer: payer.publicKey }).add(
    createAssociatedTokenAccountInstruction(
      payer.publicKey,
      associatedToken,
      owner,
      mint,
      programId,
      associatedTokenProgramId,
    ),
  );
  transaction.recentBlockhash = await (await connection.getLatestBlockhash()).blockhash;
  await payer.signTransaction(transaction);

  await sendAndConfirmRawTransaction(connection, transaction.serialize(), confirmOptions);

  return associatedToken;
}
