import fetch from 'cross-fetch';
import { PublicKey, Keypair } from '@solana/web3.js';
import { Program, Provider, Wallet, Idl, AnchorProvider, BN } from '@project-serum/anchor';
import {
  Metadata,
  PROGRAM_ID as METADATA_PROGRAM_ID,
} from '@metaplex-foundation/mpl-token-metadata';
import {
  getAccount as getTokenAccount,
  getMint,
  Account as TokenAccount,
  Mint,
} from '@solana/spl-token';
import idl from '../../../target/idl/bpl_token_metadata.json';
import { Promo, PromoExtended, DataV2, MetadataJson, AdminSettings, PromoExtendeds, Memo } from '.';
const camelcaseKeysDeep = require('camelcase-keys-deep');

export class TokenMetadataProgram {
  readonly PUBKEY: PublicKey;

  readonly SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: PublicKey;
  readonly TOKEN_PROGRAM_ID: PublicKey;
  readonly TOKEN_METADATA_PROGRAM_ID: PublicKey;
  readonly MEMO_PROGRAM_ID: PublicKey;

  readonly ADMIN_PREFIX: string;
  readonly AUTHORITY_PREFIX: string;
  readonly METADATA_PREFIX: string;
  readonly PROMO_PREFIX: string;

  program: Program;
  payer: Wallet;

  constructor(provider: Provider) {
    this.PUBKEY = new PublicKey('CjSoZrc2DBZTv1UdoMx8fTcCpqEMXCyfm2EuTwy8yiGi');
    this.SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new PublicKey(
      'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL',
    );
    this.TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
    this.TOKEN_METADATA_PROGRAM_ID = METADATA_PROGRAM_ID;
    this.MEMO_PROGRAM_ID = new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr');

    this.ADMIN_PREFIX = 'admin';
    this.AUTHORITY_PREFIX = 'authority';
    this.METADATA_PREFIX = 'metadata';
    this.PROMO_PREFIX = 'promo';

    this.program = new Program(idl as Idl, this.PUBKEY, provider);
    const anchorProvider = this.program.provider as AnchorProvider;
    this.payer = anchorProvider.wallet as Wallet;
  }

  // To keep things straight with promo owner paying for transactions
  // initiated and signed for by users, always pass and explicit
  // reference to the payer into accounts.

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
    burnPromoTokenLamports: number,
  ): Promise<PublicKey> {
    const [adminSettings] = await this.findAdminAddress();

    await this.program.methods
      .createAdminSettings({
        platform: platform.publicKey,
        createPromoLamports: new BN(createPromoLamports),
        burnPromoTokenLamports: new BN(burnPromoTokenLamports),
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
    const adminSettingsAccount = (await this.program.account.adminSettings.fetch(
      adminSettings,
    )) as AdminSettings;
    return adminSettingsAccount.platform;
  }

  /**
   * Create promo and associated metadata accounts
   *
   * @param payer         Payer of the transaction, will be the owner of the promo 
   * @param platform      Platform address
   * @param metadataData  Metadata data
   * @param isMutable     Whether metadata is mutable
   * @param maxMint       Optional Max number of tokens to mint
   * @param maxRedeemable Optional max number of tokens that can used
   *
   * @return Address of promo account
   */
  async createPromo(
    metadataData: DataV2,
    isMutable: boolean,
    maxMint: number | null,
    maxBurn: number | null,
    platform: PublicKey,
    memo: Memo | null
  ): Promise<PublicKey> {
    const mint = Keypair.generate();

    const [metadata] = await this.findMetadataAddress(mint.publicKey);

    const promoData: Promo = {
      owner: this.payer.publicKey,
      mint: mint.publicKey,
      metadata,
      mintCount: 0,
      burnCount: 0,
      maxMint,
      maxBurn,
    };

    await this.program.methods
      .createPromo(promoData, metadataData, isMutable, memo)
      .accounts({
        mint: mint.publicKey,
        metadata,
        platform,
        metadataProgram: this.TOKEN_METADATA_PROGRAM_ID,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .signers([mint])
      .rpc();

    return mint.publicKey;
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
  // no promo owner as signer for demo
  async mintPromoToken(mint: PublicKey, promoOwner: Keypair, memo: Memo | null): Promise<PublicKey> {
    const [tokenAccount] = await this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);

    await this.program.methods
      .mintPromoToken(memo)
      .accounts({
        payer: promoOwner.publicKey,
        tokenOwner: this.payer.publicKey,
        mint,
        tokenAccount,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .signers([promoOwner])
      .rpc();

    return tokenAccount;
  }

  /**
   * Delegate promo token
   *
   * @param mint  Mint address
   *
   * @return Token account address
   */
  async delegatePromoToken(mint: PublicKey, promoOwner: Keypair, memo: Memo | null): Promise<PublicKey> {
    const [tokenAccount] = await this.findAssociatedTokenAccountAddress(mint, this.payer.publicKey);

    await this.program.methods.delegatePromoToken(memo).accounts({
      payer: promoOwner.publicKey,
      tokenOwner: this.payer.publicKey,
      tokenAccount,
      memoProgram: this.MEMO_PROGRAM_ID,
    })
      .signers([promoOwner])
      .rpc();

    return tokenAccount;
  }

  /**
   * Burn promo token.
   *
   * @param platform  Platform address
   * @param mint  Mint address
   *
   * @return Token account address
   */
  // no promo owner as signer for demo
  async burnDelegatedPromoToken(
    mint: PublicKey,
    tokenOwner: PublicKey,
    platform: PublicKey,
    memo: Memo | null
  ): Promise<PublicKey> {
    const [tokenAccount] = await this.findAssociatedTokenAccountAddress(mint, tokenOwner);

    await this.program.methods
      .burnDelegatedPromoToken(memo)
      .accounts({
        mint,
        platform,
        tokenAccount,
        memoProgram: this.MEMO_PROGRAM_ID,
      })
      .rpc();

    return tokenAccount;
  }

  async getTokenAccount(address: PublicKey): Promise<TokenAccount> {
    return await getTokenAccount(this.program.provider.connection, address);
  }

  async getMintAccount(address: PublicKey): Promise<Mint> {
    return await getMint(this.program.provider.connection, address);
  }

  async getMetadataAccount(address: PublicKey): Promise<Metadata> {
    return await Metadata.fromAccountAddress(this.program.provider.connection, address);
  }

  async getPromoExtended(mint: PublicKey): Promise<PromoExtended> {
    const [[promo], [metadata]] = await Promise.all([
      this.findPromoAddress(mint),
      this.findMetadataAddress(mint),
    ]);

    const [promoAccount, mintAccount, metadataAccount] = (await Promise.all([
      this.program.account.promo.fetch(promo),
      this.getMintAccount(mint),
      this.getMetadataAccount(metadata),
    ])) as [Promo, Mint, Metadata];
    const metadataJson = camelcaseKeysDeep(
      await fetch(metadataAccount.data.uri).then((res) => {
        return res.json();
      }),
    ) as MetadataJson;
    return new PromoExtendedImpl(promo, promoAccount, mintAccount, metadata, metadataAccount, metadataJson);
  }

  async updatePromoExtended(promoExtended: PromoExtended): Promise<PromoExtended> {
    const promoAccount = (await this.program.account.promo.fetch(promoExtended.publicKey)) as Promo;
    const mintAccount = await this.getMintAccount(promoExtended.mintAccount.address);
    return new PromoExtendedImpl(
      promoExtended.publicKey,
      promoAccount,
      mintAccount,
      promoExtended.metadata,
      promoExtended.metadataAccount,
      promoExtended.metadataJson,
    );
  }

  async updatePromoExtendeds(promoExtendeds: PromoExtendeds): Promise<PromoExtendeds> {
    const results = await Promise.all(
      Object.values(promoExtendeds).map((promoExtended) => this.updatePromoExtended(promoExtended)),
    );
    return results.reduce(
      (promoExtendedsNew, promoExtended) => (
        (promoExtendedsNew[promoExtended.mintAccount.address.toString()] = promoExtended),
        promoExtendedsNew
      ),
      {} as PromoExtendeds,
    );
  }

  async getPromoExtendeds(mints: PublicKey[]): Promise<PromoExtendeds> {
    const results = await Promise.all(mints.map((mint) => this.getPromoExtended(mint)));
    return results.reduce(
      (promoExtendeds, promoExtended) => (
        (promoExtendeds[promoExtended.mintAccount.address.toString()] = promoExtended),
        promoExtendeds
      ),
      {} as PromoExtendeds,
    );
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

export class PromoExtendedImpl implements PromoExtended {
  owner: PublicKey;
  mint: PublicKey;
  metadata: PublicKey;
  mintCount: number;
  burnCount: number;
  maxMint: number | null;
  maxBurn: number | null;
  publicKey: PublicKey;
  mintAccount: Mint;
  metadataAccount: Metadata;
  metadataJson: MetadataJson;

  constructor(
    promo: PublicKey,
    promoAccount: Promo,
    mintAccount: Mint,
    metadata: PublicKey,
    metadataAccount: Metadata,
    metadataJson: MetadataJson,
  ) {
    this.owner = promoAccount.owner;
    this.publicKey = promo;
    this.mint = mintAccount.address;
    this.metadata = metadata;
    this.mintAccount = mintAccount;
    this.metadataAccount = metadataAccount;
    this.metadataJson = metadataJson;
    this.mintCount = promoAccount.mintCount;
    this.burnCount = promoAccount.burnCount;
    this.maxMint = promoAccount.maxMint;
    this.maxBurn = promoAccount.maxBurn;

  }
}
