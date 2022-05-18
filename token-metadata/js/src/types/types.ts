import { BN } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { PromoExtended } from '../';

export type Network =
  | 'http://127.0.0.1:8899'
  | 'https://api.devnet.solana.com'
  | 'https://api.mainnet-beta.solana.com';

export type AdminSettings = {
  platform: PublicKey;
  createPromoLamports: BN;
  burnPromoTokenLamports: BN;
};

export type UI<T> = T & {
  publicKey: PublicKey;
};

export type Promo = {
  owner: PublicKey;
  mint: PublicKey;
  metadata: PublicKey;
  mints: number;
  burns: number;
  maxMint: number | null;
  maxBurn: number | null;
  expiry: BN | null;
};

export type PromoExtendeds = {
  [key: string]: PromoExtended;
};

export type Promos = {
  [key: string]: UI<Promo>;
};

export type AccountBalance = {
  key: PublicKey;
  lamports: number;
};

export type DataV2 = {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number;
  creators: Creator[] | null;
  collection: Collection | null;
  uses: Uses | null;
};

export type Creator = {
  address: PublicKey;
  verified: boolean;
  share: number;
};

export type Collection = {
  verified: boolean;
  key: PublicKey;
};

export type UseMethod = { burn: {} } | { multiple: {} } | { single: {} };

export type Uses = {
  useMethod: UseMethod;
  remaining: number;
  total: number;
};

export type Attribute = {
  traitType: string | number;
  value: string | number;
};

export type CreatorJson = {
  address: string;
  share: number;
};

export type MetadataJson = {
  name: string;
  symbol?: string;
  description?: string;
  sellerFeeBasisPoints: number;
  image: string;
  animationUrl?: string;
  externalUrl?: string;
  attributes?: Attribute[];
  collection: {
    name: string;
    family: string;
  };
  properties: {
    files: {
      uri: string;
      type: 'image/png' | 'video/mp4';
    }[];
    category: 'video' | 'image';
    creators?: CreatorJson[];
  };
};
