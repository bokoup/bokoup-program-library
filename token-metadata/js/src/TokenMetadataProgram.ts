import {
  PublicKey,
  ConfirmOptions,
  Keypair,
  Connection,
  Transaction,
  sendAndConfirmTransaction,
  sendAndConfirmRawTransaction,
  SYSVAR_INSTRUCTIONS_PUBKEY
} from '@solana/web3.js';
import { Program, Provider, Wallet } from '@project-serum/anchor';

import { Promo, DataV2 } from '.';
import { Metadata, PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';
import idl from '../../../target/idl/bpl_token_metadata.json';


export class TokenMetadataProgram {
  static readonly PUBKEY = new PublicKey('CsmkSwyBPpihA6qiNGKtWR3DV6RNxJKBo4xBMPt414Eq');

  static readonly ADMIN_PREFIX = 'admin';
  static readonly AUTHORITY_PREFIX = 'authority';
  static readonly PROMO_PREFIX = 'promo';

  static readonly TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
  static readonly SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
  );

  static readonly METADATA_PREFIX = 'metadata';
  static readonly TOKEN_METADATA_PROGRAM_ID = new PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
  );

  static readonly confirmOptions: ConfirmOptions = {
    skipPreflight: false,
    commitment: 'confirmed',
    preflightCommitment: 'processed',
    maxRetries: 10,
  };

  static getProgram(provider: Provider): Program {
    // @ts-ignore
    return new Program(idl, this.PUBKEY, provider);
  }

  /**
  * Return platform address, creating admin settings account if necessary
  *
  * @param program  Anchor program to use
  * @param keypair  Payer of the transaction and initialization fees
  *
  * @return Address of the platform account
  */
  static async getPlatformAddress(program: Program, keypair?: Keypair): Promise<[PublicKey, null] | [PublicKey, Keypair]> {
    const [adminSettings] = await TokenMetadataProgram.findAdminAddress();
    // const [programData] = await PublicKey.findProgramAddress([
    //   program.programId.toBuffer()
    // ], new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111"));

    let adminSettingsAccount;
    // let keypair;
    try {
      adminSettingsAccount = await program.account.adminSettings.fetch(adminSettings);
      return adminSettingsAccount.platform
    } catch {
      if (keypair === undefined) {
        keypair = Keypair.generate();
      }
      const data = {
        platform: keypair.publicKey
      }
      await program.methods
        .createAdminSettings(data)
        .accounts({
          adminSettings,
          // program: program.programId,
          // programData
        })
        .rpc();
      return [keypair.publicKey, keypair]
    }
  }

  /**
  * Create promo and associated metadata accounts
  *
  * @param program           Anchor program to use
  * @param promoData         Promo data
  * @param metadataData      Metdata data
  * @param isMutable         Whether metadata is mutable
  * @param mint              Mint to use for promo
  *
  * @return Address of promo account
  */
  static async createPromo(
    program: Program,
    promoData: Promo,
    metadataData: DataV2,
    isMutable: boolean,
    mint: Keypair,
    platform: PublicKey
  ): Promise<PublicKey> {

    const [[authority], [promo], [metadata], [adminSettings]] = await Promise.all([
      TokenMetadataProgram.findAuthorityAddress(),
      TokenMetadataProgram.findPromoAddress(mint.publicKey),
      TokenMetadataProgram.findMetadataAddress(mint.publicKey),
      TokenMetadataProgram.findAdminAddress()
    ]);

    promoData.metadata = metadata

    await program.methods
      .createPromo(promoData, metadataData, isMutable)
      .accounts({
        mint: mint.publicKey,
        metadata,
        platform,
        adminSettings,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .signers([mint])
      .rpc();

    return promo

  }

  static async findAssociatedTokenAccountAddress(
    mint: PublicKey,
    wallet: PublicKey,
  ): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [wallet.toBuffer(), this.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    );
  }

  static async findAdminAddress(): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from(this.ADMIN_PREFIX)],
      this.PUBKEY,
    );
  }

  static async findAuthorityAddress(): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from(this.AUTHORITY_PREFIX)],
      this.PUBKEY,
    );
  }

  static async findMetadataAddress(mint: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [
        Buffer.from(this.METADATA_PREFIX),
        this.TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      this.TOKEN_METADATA_PROGRAM_ID,
    );
  }

  static async findPromoAddress(mint: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from(this.PROMO_PREFIX), mint.toBuffer()],
      this.PUBKEY,
    );
  }


}
