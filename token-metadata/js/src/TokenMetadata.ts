import { PublicKey, Keypair } from '@solana/web3.js';
import { Program, Provider, Wallet, Idl, AnchorProvider, BN } from '@project-serum/anchor';
import { Promo, DataV2 } from '.';
import { PROGRAM_ID as METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata';
import {
  getAccount,
  getMint as getTokenMint,
  Account as TokenAccount,
  Mint as Mint,
} from '@solana/spl-token';
import idl from '../../../target/idl/bpl_token_metadata.json';

export class TokenMetadata {
  readonly PUBKEY: PublicKey;

  readonly SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey;
  readonly TOKEN_PROGRAM_ID: PublicKey;
  readonly TOKEN_METADATA_PROGRAM_ID: PublicKey;

  readonly ADMIN_PREFIX: string;
  readonly AUTHORITY_PREFIX: string;
  readonly METADATA_PREFIX: string;
  readonly PROMO_PREFIX: string;

  program: Program;
  payer: Wallet;

  constructor(provider: Provider) {
    this.PUBKEY = new PublicKey('CsmkSwyBPpihA6qiNGKtWR3DV6RNxJKBo4xBMPt414Eq');
    this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
      'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
    );
    this.TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
    this.TOKEN_METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

    this.ADMIN_PREFIX = 'admin';
    this.AUTHORITY_PREFIX = 'authority';
    this.METADATA_PREFIX = 'metadata';
    this.PROMO_PREFIX = 'promo';

    this.program = new Program(idl as Idl, this.PUBKEY, provider);
    const anchorProvider = this.program.provider as AnchorProvider;
    this.payer = anchorProvider.wallet as Wallet;
  }

  /**
   * Creates admin settings account
   *
   * @param platform  Payer of the transaction and initialization fees
   *
   * @return Address of the admin settings account
   */
  async createAdminSettings(
    platform: Keypair,
    createPromoLamports: number,
    redeemPromoTokenLamports: number,
  ): Promise<PublicKey> {
    const [adminSettings] = await this.findAdminAddress();

    await this.program.methods
      .createAdminSettings({
        platform: platform.publicKey,
        createPromoLamports: new BN(createPromoLamports),
        redeemPromoTokenLamports: new BN(redeemPromoTokenLamports),
      })
      .accounts({
        payer: platform.publicKey,
      })
      .signers([platform])
      .rpc();
    return adminSettings;
  }

  /**
   * Fetch platform address
   *
   * @return Address of the platform account
   */
  async fetchPlatformAddress(): Promise<PublicKey> {
    const [adminSettings] = await this.findAdminAddress();
    const adminSettingsAccount = await this.program.account.adminSettings.fetch(adminSettings);
    return adminSettingsAccount.platform;
  }

  /**
   * Create promo and associated metadata accounts
   *
   * @param platform     Platform address
   * @param metadataData Metadata data
   * @param isMutable    Whether metadata is mutable
   * @param maxMint      Max number of tokens to mint
   * @param maxBurn      Optional max number of tokens that can used
   * @param expiry       Optional expiration date
   * @param payer        Optional alternate owner and payer
   *
   * @return Address of promo account
   */
  async createPromo(
    platform: PublicKey,
    metadataData: DataV2,
    isMutable: boolean,
    maxMint: number,
    maxBurn?: number,
    expiry?: Date,
    payer?: Keypair,
  ): Promise<PublicKey> {
    const mint = Keypair.generate();

    const [[promo], [metadata]] = await Promise.all([
      this.findPromoAddress(mint.publicKey),
      this.findMetadataAddress(mint.publicKey),
    ]);

    const signers = [mint];
    let owner = this.payer.publicKey;
    if (payer != undefined) {
      owner = payer.publicKey;
      signers.push(payer);
    }

    const promoData: Promo = {
      owner,
      mint: mint.publicKey,
      metadata,
      mints: 0,
      burns: 0,
      maxMint,
      maxBurn: maxBurn == undefined ? null : maxBurn,
      expiry: expiry == undefined ? null : new BN(expiry.valueOf() / 1000),
    };

    promoData.metadata = metadata;

    await this.program.methods
      .createPromo(promoData, metadataData, isMutable)
      .accounts({
        payer: owner,
        mint: mint.publicKey,
        metadata,
        platform,
        metadataProgram: METADATA_PROGRAM_ID,
      })
      .signers(signers)
      .rpc();

    return promo;
  }

  /**
   * Mint promo token
   *
   * @param mint       Promo mint
   * @param platform   Address of platform account
   * @param promoOwner Keypair of promo owner
   *
   * @return Address of promo account
   */
  async mintPromoToken(mint: PublicKey, promoOwner: Keypair): Promise<PublicKey> {
    const [tokenAccount] = await this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);

    await this.program.methods
      .mintPromoToken()
      .accounts({
        mint,
        tokenAccount,
        promoOwner: promoOwner.publicKey,
      })
      .signers([promoOwner])
      .rpc();

    return tokenAccount;
  }

  /**
   * Delegate promo token
   *
   * @param promo Promo address
   * @param mint  Mint address
   *
   * @return Token account address
   */
  async delegatePromoToken(promo: PublicKey, mint: PublicKey): Promise<PublicKey> {
    const [tokenAccount] = await this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);

    await this.program.methods
      .delegatePromoToken()
      .accounts({
        promo,
        tokenAccount,
      })
      .rpc();

    return tokenAccount;
  }

  /**
   * Burn promo token
   *
   * @param promo Promo address
   * @param mint  Mint address
   *
   * @return Token account address
   */
  async burnPromoToken(mint: PublicKey, promoOwner: Keypair): Promise<PublicKey> {
    const [tokenAccount] = await this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);

    await this.program.methods
      .burnPromoToken()
      .accounts({
        mint,
        tokenAccount,
        promoOwner: promoOwner.publicKey,
      })
      .signers([promoOwner])
      .rpc();

    return tokenAccount;
  }

  async getTokenAccount(address: PublicKey): Promise<TokenAccount> {
    return await getAccount(this.program.provider.connection, address);
  }

  async getMint(address: PublicKey): Promise<Mint> {
    return await getTokenMint(this.program.provider.connection, address);
  }

  async findAssociatedTokenAccountAddress(
    mint: PublicKey,
    wallet: PublicKey,
  ): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [wallet.toBuffer(), this.TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
      this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    );
  }

  async findAdminAddress(): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress([Buffer.from(this.ADMIN_PREFIX)], this.PUBKEY);
  }

  async findAuthorityAddress(): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress([Buffer.from(this.AUTHORITY_PREFIX)], this.PUBKEY);
  }

  async findMetadataAddress(mint: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [
        Buffer.from(this.METADATA_PREFIX),
        this.TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      this.TOKEN_METADATA_PROGRAM_ID,
    );
  }

  async findPromoAddress(mint: PublicKey): Promise<[PublicKey, number]> {
    return await PublicKey.findProgramAddress(
      [Buffer.from(this.PROMO_PREFIX), mint.toBuffer()],
      this.PUBKEY,
    );
  }
}
