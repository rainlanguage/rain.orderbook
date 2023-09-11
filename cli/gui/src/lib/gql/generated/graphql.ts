/* eslint-disable */
import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  BigDecimal: any;
  BigInt: any;
  Bytes: any;
  /**
   * 8 bytes signed integer
   *
   */
  Int8: any;
};

export type Account = {
  __typename?: 'Account';
  bounties?: Maybe<Array<Bounty>>;
  deposits?: Maybe<Array<VaultDeposit>>;
  events?: Maybe<Array<Event>>;
  id: Scalars['Bytes'];
  orders?: Maybe<Array<Order>>;
  ordersCleared?: Maybe<Array<OrderClear>>;
  takeOrderEntities?: Maybe<Array<TakeOrderEntity>>;
  tokenVaults?: Maybe<Array<TokenVault>>;
  vaults?: Maybe<Array<Vault>>;
  withdraws?: Maybe<Array<VaultWithdraw>>;
};


export type AccountBountiesArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Bounty_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Bounty_Filter>;
};


export type AccountDepositsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultDeposit_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<VaultDeposit_Filter>;
};


export type AccountEventsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Event_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Event_Filter>;
};


export type AccountOrdersArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Order_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Order_Filter>;
};


export type AccountOrdersClearedArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClear_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<OrderClear_Filter>;
};


export type AccountTakeOrderEntitiesArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TakeOrderEntity_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<TakeOrderEntity_Filter>;
};


export type AccountTokenVaultsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TokenVault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<TokenVault_Filter>;
};


export type AccountVaultsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Vault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Vault_Filter>;
};


export type AccountWithdrawsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultWithdraw_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<VaultWithdraw_Filter>;
};

export type Account_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Account_Filter>>>;
  bounties_?: InputMaybe<Bounty_Filter>;
  deposits_?: InputMaybe<VaultDeposit_Filter>;
  events_?: InputMaybe<Event_Filter>;
  id?: InputMaybe<Scalars['Bytes']>;
  id_contains?: InputMaybe<Scalars['Bytes']>;
  id_gt?: InputMaybe<Scalars['Bytes']>;
  id_gte?: InputMaybe<Scalars['Bytes']>;
  id_in?: InputMaybe<Array<Scalars['Bytes']>>;
  id_lt?: InputMaybe<Scalars['Bytes']>;
  id_lte?: InputMaybe<Scalars['Bytes']>;
  id_not?: InputMaybe<Scalars['Bytes']>;
  id_not_contains?: InputMaybe<Scalars['Bytes']>;
  id_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  or?: InputMaybe<Array<InputMaybe<Account_Filter>>>;
  ordersCleared_?: InputMaybe<OrderClear_Filter>;
  orders_?: InputMaybe<Order_Filter>;
  takeOrderEntities_?: InputMaybe<TakeOrderEntity_Filter>;
  tokenVaults_?: InputMaybe<TokenVault_Filter>;
  vaults_?: InputMaybe<Vault_Filter>;
  withdraws_?: InputMaybe<VaultWithdraw_Filter>;
};

export type Account_OrderBy =
  | 'bounties'
  | 'deposits'
  | 'events'
  | 'id'
  | 'orders'
  | 'ordersCleared'
  | 'takeOrderEntities'
  | 'tokenVaults'
  | 'vaults'
  | 'withdraws';

export type BlockChangedFilter = {
  number_gte: Scalars['Int'];
};

export type Block_Height = {
  hash?: InputMaybe<Scalars['Bytes']>;
  number?: InputMaybe<Scalars['Int']>;
  number_gte?: InputMaybe<Scalars['Int']>;
};

export type Bounty = {
  __typename?: 'Bounty';
  /** The amount paid for bounty token A */
  bountyAmountA?: Maybe<Scalars['BigInt']>;
  bountyAmountADisplay?: Maybe<Scalars['BigDecimal']>;
  /** The amount paid for bounty token B */
  bountyAmountB?: Maybe<Scalars['BigInt']>;
  bountyAmountBDisplay?: Maybe<Scalars['BigDecimal']>;
  /** The A token for the bounty */
  bountyTokenA: Erc20;
  /** The B token for the bounty */
  bountyTokenB: Erc20;
  /** The Vault that bounty token A was deposited into */
  bountyVaultA: Vault;
  /** The Vault that bounty token B was deposited into */
  bountyVaultB: Vault;
  /** The clearer who received this bounty */
  clearer: Account;
  emitter: Account;
  id: Scalars['ID'];
  /** The Clear event that paid this bounty */
  orderClear: OrderClear;
  timestamp: Scalars['BigInt'];
  transaction: Transaction;
};

export type Bounty_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Bounty_Filter>>>;
  bountyAmountA?: InputMaybe<Scalars['BigInt']>;
  bountyAmountADisplay?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountADisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountADisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountADisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  bountyAmountADisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountADisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountADisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountADisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  bountyAmountA_gt?: InputMaybe<Scalars['BigInt']>;
  bountyAmountA_gte?: InputMaybe<Scalars['BigInt']>;
  bountyAmountA_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bountyAmountA_lt?: InputMaybe<Scalars['BigInt']>;
  bountyAmountA_lte?: InputMaybe<Scalars['BigInt']>;
  bountyAmountA_not?: InputMaybe<Scalars['BigInt']>;
  bountyAmountA_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bountyAmountB?: InputMaybe<Scalars['BigInt']>;
  bountyAmountBDisplay?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountBDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountBDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountBDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  bountyAmountBDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountBDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountBDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  bountyAmountBDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  bountyAmountB_gt?: InputMaybe<Scalars['BigInt']>;
  bountyAmountB_gte?: InputMaybe<Scalars['BigInt']>;
  bountyAmountB_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bountyAmountB_lt?: InputMaybe<Scalars['BigInt']>;
  bountyAmountB_lte?: InputMaybe<Scalars['BigInt']>;
  bountyAmountB_not?: InputMaybe<Scalars['BigInt']>;
  bountyAmountB_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bountyTokenA?: InputMaybe<Scalars['String']>;
  bountyTokenA_?: InputMaybe<Erc20_Filter>;
  bountyTokenA_contains?: InputMaybe<Scalars['String']>;
  bountyTokenA_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenA_ends_with?: InputMaybe<Scalars['String']>;
  bountyTokenA_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenA_gt?: InputMaybe<Scalars['String']>;
  bountyTokenA_gte?: InputMaybe<Scalars['String']>;
  bountyTokenA_in?: InputMaybe<Array<Scalars['String']>>;
  bountyTokenA_lt?: InputMaybe<Scalars['String']>;
  bountyTokenA_lte?: InputMaybe<Scalars['String']>;
  bountyTokenA_not?: InputMaybe<Scalars['String']>;
  bountyTokenA_not_contains?: InputMaybe<Scalars['String']>;
  bountyTokenA_not_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenA_not_ends_with?: InputMaybe<Scalars['String']>;
  bountyTokenA_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenA_not_in?: InputMaybe<Array<Scalars['String']>>;
  bountyTokenA_not_starts_with?: InputMaybe<Scalars['String']>;
  bountyTokenA_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenA_starts_with?: InputMaybe<Scalars['String']>;
  bountyTokenA_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenB?: InputMaybe<Scalars['String']>;
  bountyTokenB_?: InputMaybe<Erc20_Filter>;
  bountyTokenB_contains?: InputMaybe<Scalars['String']>;
  bountyTokenB_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenB_ends_with?: InputMaybe<Scalars['String']>;
  bountyTokenB_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenB_gt?: InputMaybe<Scalars['String']>;
  bountyTokenB_gte?: InputMaybe<Scalars['String']>;
  bountyTokenB_in?: InputMaybe<Array<Scalars['String']>>;
  bountyTokenB_lt?: InputMaybe<Scalars['String']>;
  bountyTokenB_lte?: InputMaybe<Scalars['String']>;
  bountyTokenB_not?: InputMaybe<Scalars['String']>;
  bountyTokenB_not_contains?: InputMaybe<Scalars['String']>;
  bountyTokenB_not_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenB_not_ends_with?: InputMaybe<Scalars['String']>;
  bountyTokenB_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenB_not_in?: InputMaybe<Array<Scalars['String']>>;
  bountyTokenB_not_starts_with?: InputMaybe<Scalars['String']>;
  bountyTokenB_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyTokenB_starts_with?: InputMaybe<Scalars['String']>;
  bountyTokenB_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultA?: InputMaybe<Scalars['String']>;
  bountyVaultA_?: InputMaybe<Vault_Filter>;
  bountyVaultA_contains?: InputMaybe<Scalars['String']>;
  bountyVaultA_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultA_ends_with?: InputMaybe<Scalars['String']>;
  bountyVaultA_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultA_gt?: InputMaybe<Scalars['String']>;
  bountyVaultA_gte?: InputMaybe<Scalars['String']>;
  bountyVaultA_in?: InputMaybe<Array<Scalars['String']>>;
  bountyVaultA_lt?: InputMaybe<Scalars['String']>;
  bountyVaultA_lte?: InputMaybe<Scalars['String']>;
  bountyVaultA_not?: InputMaybe<Scalars['String']>;
  bountyVaultA_not_contains?: InputMaybe<Scalars['String']>;
  bountyVaultA_not_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultA_not_ends_with?: InputMaybe<Scalars['String']>;
  bountyVaultA_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultA_not_in?: InputMaybe<Array<Scalars['String']>>;
  bountyVaultA_not_starts_with?: InputMaybe<Scalars['String']>;
  bountyVaultA_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultA_starts_with?: InputMaybe<Scalars['String']>;
  bountyVaultA_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultB?: InputMaybe<Scalars['String']>;
  bountyVaultB_?: InputMaybe<Vault_Filter>;
  bountyVaultB_contains?: InputMaybe<Scalars['String']>;
  bountyVaultB_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultB_ends_with?: InputMaybe<Scalars['String']>;
  bountyVaultB_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultB_gt?: InputMaybe<Scalars['String']>;
  bountyVaultB_gte?: InputMaybe<Scalars['String']>;
  bountyVaultB_in?: InputMaybe<Array<Scalars['String']>>;
  bountyVaultB_lt?: InputMaybe<Scalars['String']>;
  bountyVaultB_lte?: InputMaybe<Scalars['String']>;
  bountyVaultB_not?: InputMaybe<Scalars['String']>;
  bountyVaultB_not_contains?: InputMaybe<Scalars['String']>;
  bountyVaultB_not_contains_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultB_not_ends_with?: InputMaybe<Scalars['String']>;
  bountyVaultB_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultB_not_in?: InputMaybe<Array<Scalars['String']>>;
  bountyVaultB_not_starts_with?: InputMaybe<Scalars['String']>;
  bountyVaultB_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bountyVaultB_starts_with?: InputMaybe<Scalars['String']>;
  bountyVaultB_starts_with_nocase?: InputMaybe<Scalars['String']>;
  clearer?: InputMaybe<Scalars['String']>;
  clearer_?: InputMaybe<Account_Filter>;
  clearer_contains?: InputMaybe<Scalars['String']>;
  clearer_contains_nocase?: InputMaybe<Scalars['String']>;
  clearer_ends_with?: InputMaybe<Scalars['String']>;
  clearer_ends_with_nocase?: InputMaybe<Scalars['String']>;
  clearer_gt?: InputMaybe<Scalars['String']>;
  clearer_gte?: InputMaybe<Scalars['String']>;
  clearer_in?: InputMaybe<Array<Scalars['String']>>;
  clearer_lt?: InputMaybe<Scalars['String']>;
  clearer_lte?: InputMaybe<Scalars['String']>;
  clearer_not?: InputMaybe<Scalars['String']>;
  clearer_not_contains?: InputMaybe<Scalars['String']>;
  clearer_not_contains_nocase?: InputMaybe<Scalars['String']>;
  clearer_not_ends_with?: InputMaybe<Scalars['String']>;
  clearer_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  clearer_not_in?: InputMaybe<Array<Scalars['String']>>;
  clearer_not_starts_with?: InputMaybe<Scalars['String']>;
  clearer_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  clearer_starts_with?: InputMaybe<Scalars['String']>;
  clearer_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<Bounty_Filter>>>;
  orderClear?: InputMaybe<Scalars['String']>;
  orderClear_?: InputMaybe<OrderClear_Filter>;
  orderClear_contains?: InputMaybe<Scalars['String']>;
  orderClear_contains_nocase?: InputMaybe<Scalars['String']>;
  orderClear_ends_with?: InputMaybe<Scalars['String']>;
  orderClear_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderClear_gt?: InputMaybe<Scalars['String']>;
  orderClear_gte?: InputMaybe<Scalars['String']>;
  orderClear_in?: InputMaybe<Array<Scalars['String']>>;
  orderClear_lt?: InputMaybe<Scalars['String']>;
  orderClear_lte?: InputMaybe<Scalars['String']>;
  orderClear_not?: InputMaybe<Scalars['String']>;
  orderClear_not_contains?: InputMaybe<Scalars['String']>;
  orderClear_not_contains_nocase?: InputMaybe<Scalars['String']>;
  orderClear_not_ends_with?: InputMaybe<Scalars['String']>;
  orderClear_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderClear_not_in?: InputMaybe<Array<Scalars['String']>>;
  orderClear_not_starts_with?: InputMaybe<Scalars['String']>;
  orderClear_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderClear_starts_with?: InputMaybe<Scalars['String']>;
  orderClear_starts_with_nocase?: InputMaybe<Scalars['String']>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type Bounty_OrderBy =
  | 'bountyAmountA'
  | 'bountyAmountADisplay'
  | 'bountyAmountB'
  | 'bountyAmountBDisplay'
  | 'bountyTokenA'
  | 'bountyTokenA__decimals'
  | 'bountyTokenA__id'
  | 'bountyTokenA__name'
  | 'bountyTokenA__symbol'
  | 'bountyTokenA__totalSupply'
  | 'bountyTokenA__totalSupplyDisplay'
  | 'bountyTokenB'
  | 'bountyTokenB__decimals'
  | 'bountyTokenB__id'
  | 'bountyTokenB__name'
  | 'bountyTokenB__symbol'
  | 'bountyTokenB__totalSupply'
  | 'bountyTokenB__totalSupplyDisplay'
  | 'bountyVaultA'
  | 'bountyVaultA__id'
  | 'bountyVaultA__vaultId'
  | 'bountyVaultB'
  | 'bountyVaultB__id'
  | 'bountyVaultB__vaultId'
  | 'clearer'
  | 'clearer__id'
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'orderClear'
  | 'orderClear__aInputIOIndex'
  | 'orderClear__aOutputIOIndex'
  | 'orderClear__bInputIOIndex'
  | 'orderClear__bOutputIOIndex'
  | 'orderClear__id'
  | 'orderClear__timestamp'
  | 'timestamp'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp';

export type ClearOrderConfig = {
  __typename?: 'ClearOrderConfig';
  aliceTokenVaultInput: Scalars['String'];
  aliceTokenVaultOutput: Scalars['String'];
  bobTokenVaultInput: Scalars['String'];
  bobTokenVaultOutput: Scalars['String'];
  id: Scalars['ID'];
  orderClearId: OrderClear;
  tokenVaultBountyAlice: TokenVault;
  tokenVaultBountyBob: TokenVault;
};

export type ClearOrderConfig_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  aliceTokenVaultInput?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_contains?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_contains_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_ends_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_ends_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_gt?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_gte?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_in?: InputMaybe<Array<Scalars['String']>>;
  aliceTokenVaultInput_lt?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_lte?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not_contains?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not_contains_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not_ends_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not_in?: InputMaybe<Array<Scalars['String']>>;
  aliceTokenVaultInput_not_starts_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_starts_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultInput_starts_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_contains?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_contains_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_ends_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_ends_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_gt?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_gte?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_in?: InputMaybe<Array<Scalars['String']>>;
  aliceTokenVaultOutput_lt?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_lte?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not_contains?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not_contains_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not_ends_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not_in?: InputMaybe<Array<Scalars['String']>>;
  aliceTokenVaultOutput_not_starts_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_starts_with?: InputMaybe<Scalars['String']>;
  aliceTokenVaultOutput_starts_with_nocase?: InputMaybe<Scalars['String']>;
  and?: InputMaybe<Array<InputMaybe<ClearOrderConfig_Filter>>>;
  bobTokenVaultInput?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_contains?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_contains_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_ends_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_gt?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_gte?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_in?: InputMaybe<Array<Scalars['String']>>;
  bobTokenVaultInput_lt?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_lte?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not_contains?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not_contains_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not_ends_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not_in?: InputMaybe<Array<Scalars['String']>>;
  bobTokenVaultInput_not_starts_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_starts_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultInput_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_contains?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_contains_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_ends_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_gt?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_gte?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_in?: InputMaybe<Array<Scalars['String']>>;
  bobTokenVaultOutput_lt?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_lte?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not_contains?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not_contains_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not_ends_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not_in?: InputMaybe<Array<Scalars['String']>>;
  bobTokenVaultOutput_not_starts_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_starts_with?: InputMaybe<Scalars['String']>;
  bobTokenVaultOutput_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<ClearOrderConfig_Filter>>>;
  orderClearId?: InputMaybe<Scalars['String']>;
  orderClearId_?: InputMaybe<OrderClear_Filter>;
  orderClearId_contains?: InputMaybe<Scalars['String']>;
  orderClearId_contains_nocase?: InputMaybe<Scalars['String']>;
  orderClearId_ends_with?: InputMaybe<Scalars['String']>;
  orderClearId_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderClearId_gt?: InputMaybe<Scalars['String']>;
  orderClearId_gte?: InputMaybe<Scalars['String']>;
  orderClearId_in?: InputMaybe<Array<Scalars['String']>>;
  orderClearId_lt?: InputMaybe<Scalars['String']>;
  orderClearId_lte?: InputMaybe<Scalars['String']>;
  orderClearId_not?: InputMaybe<Scalars['String']>;
  orderClearId_not_contains?: InputMaybe<Scalars['String']>;
  orderClearId_not_contains_nocase?: InputMaybe<Scalars['String']>;
  orderClearId_not_ends_with?: InputMaybe<Scalars['String']>;
  orderClearId_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderClearId_not_in?: InputMaybe<Array<Scalars['String']>>;
  orderClearId_not_starts_with?: InputMaybe<Scalars['String']>;
  orderClearId_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderClearId_starts_with?: InputMaybe<Scalars['String']>;
  orderClearId_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_?: InputMaybe<TokenVault_Filter>;
  tokenVaultBountyAlice_contains?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_ends_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_gt?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_gte?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVaultBountyAlice_lt?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_lte?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not_contains?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not_ends_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVaultBountyAlice_not_starts_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_starts_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyAlice_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_?: InputMaybe<TokenVault_Filter>;
  tokenVaultBountyBob_contains?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_ends_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_gt?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_gte?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVaultBountyBob_lt?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_lte?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not_contains?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not_ends_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVaultBountyBob_not_starts_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_starts_with?: InputMaybe<Scalars['String']>;
  tokenVaultBountyBob_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type ClearOrderConfig_OrderBy =
  | 'aliceTokenVaultInput'
  | 'aliceTokenVaultOutput'
  | 'bobTokenVaultInput'
  | 'bobTokenVaultOutput'
  | 'id'
  | 'orderClearId'
  | 'orderClearId__aInputIOIndex'
  | 'orderClearId__aOutputIOIndex'
  | 'orderClearId__bInputIOIndex'
  | 'orderClearId__bOutputIOIndex'
  | 'orderClearId__id'
  | 'orderClearId__timestamp'
  | 'tokenVaultBountyAlice'
  | 'tokenVaultBountyAlice__balance'
  | 'tokenVaultBountyAlice__balanceDisplay'
  | 'tokenVaultBountyAlice__id'
  | 'tokenVaultBountyAlice__vaultId'
  | 'tokenVaultBountyBob'
  | 'tokenVaultBountyBob__balance'
  | 'tokenVaultBountyBob__balanceDisplay'
  | 'tokenVaultBountyBob__id'
  | 'tokenVaultBountyBob__vaultId';

export type ContextEntity = Event & {
  __typename?: 'ContextEntity';
  /** Contains the DECIMAL RESCALED calculations */
  calculationsContext?: Maybe<Array<Scalars['BigInt']>>;
  /** Base caller */
  caller: Account;
  /** Contextual data available to both calculate order and handle IO */
  callingContext?: Maybe<Array<Scalars['BigInt']>>;
  /** Base contract */
  contract: OrderBook;
  /** Account that sent the transaction this event was emitted in. */
  emitter: Account;
  id: Scalars['ID'];
  /** Optional signed context relevant to the transaction */
  signedContext?: Maybe<Array<SignedContext>>;
  timestamp: Scalars['BigInt'];
  /** Transaction where this event was emitted. */
  transaction: Transaction;
  /** The inputs context data */
  vaultInputsContext?: Maybe<Array<Scalars['BigInt']>>;
  /** The outputs context data */
  vaultOutputsContext?: Maybe<Array<Scalars['BigInt']>>;
};


export type ContextEntitySignedContextArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<SignedContext_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<SignedContext_Filter>;
};

export type ContextEntity_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<ContextEntity_Filter>>>;
  calculationsContext?: InputMaybe<Array<Scalars['BigInt']>>;
  calculationsContext_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  calculationsContext_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  calculationsContext_not?: InputMaybe<Array<Scalars['BigInt']>>;
  calculationsContext_not_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  calculationsContext_not_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  caller?: InputMaybe<Scalars['String']>;
  caller_?: InputMaybe<Account_Filter>;
  caller_contains?: InputMaybe<Scalars['String']>;
  caller_contains_nocase?: InputMaybe<Scalars['String']>;
  caller_ends_with?: InputMaybe<Scalars['String']>;
  caller_ends_with_nocase?: InputMaybe<Scalars['String']>;
  caller_gt?: InputMaybe<Scalars['String']>;
  caller_gte?: InputMaybe<Scalars['String']>;
  caller_in?: InputMaybe<Array<Scalars['String']>>;
  caller_lt?: InputMaybe<Scalars['String']>;
  caller_lte?: InputMaybe<Scalars['String']>;
  caller_not?: InputMaybe<Scalars['String']>;
  caller_not_contains?: InputMaybe<Scalars['String']>;
  caller_not_contains_nocase?: InputMaybe<Scalars['String']>;
  caller_not_ends_with?: InputMaybe<Scalars['String']>;
  caller_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  caller_not_in?: InputMaybe<Array<Scalars['String']>>;
  caller_not_starts_with?: InputMaybe<Scalars['String']>;
  caller_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  caller_starts_with?: InputMaybe<Scalars['String']>;
  caller_starts_with_nocase?: InputMaybe<Scalars['String']>;
  callingContext?: InputMaybe<Array<Scalars['BigInt']>>;
  callingContext_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  callingContext_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  callingContext_not?: InputMaybe<Array<Scalars['BigInt']>>;
  callingContext_not_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  callingContext_not_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  contract?: InputMaybe<Scalars['String']>;
  contract_?: InputMaybe<OrderBook_Filter>;
  contract_contains?: InputMaybe<Scalars['String']>;
  contract_contains_nocase?: InputMaybe<Scalars['String']>;
  contract_ends_with?: InputMaybe<Scalars['String']>;
  contract_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contract_gt?: InputMaybe<Scalars['String']>;
  contract_gte?: InputMaybe<Scalars['String']>;
  contract_in?: InputMaybe<Array<Scalars['String']>>;
  contract_lt?: InputMaybe<Scalars['String']>;
  contract_lte?: InputMaybe<Scalars['String']>;
  contract_not?: InputMaybe<Scalars['String']>;
  contract_not_contains?: InputMaybe<Scalars['String']>;
  contract_not_contains_nocase?: InputMaybe<Scalars['String']>;
  contract_not_ends_with?: InputMaybe<Scalars['String']>;
  contract_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contract_not_in?: InputMaybe<Array<Scalars['String']>>;
  contract_not_starts_with?: InputMaybe<Scalars['String']>;
  contract_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  contract_starts_with?: InputMaybe<Scalars['String']>;
  contract_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<ContextEntity_Filter>>>;
  signedContext?: InputMaybe<Array<Scalars['String']>>;
  signedContext_?: InputMaybe<SignedContext_Filter>;
  signedContext_contains?: InputMaybe<Array<Scalars['String']>>;
  signedContext_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  signedContext_not?: InputMaybe<Array<Scalars['String']>>;
  signedContext_not_contains?: InputMaybe<Array<Scalars['String']>>;
  signedContext_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vaultInputsContext?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultInputsContext_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultInputsContext_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultInputsContext_not?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultInputsContext_not_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultInputsContext_not_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultOutputsContext?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultOutputsContext_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultOutputsContext_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultOutputsContext_not?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultOutputsContext_not_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultOutputsContext_not_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
};

export type ContextEntity_OrderBy =
  | 'calculationsContext'
  | 'caller'
  | 'caller__id'
  | 'callingContext'
  | 'contract'
  | 'contract__address'
  | 'contract__deployer'
  | 'contract__id'
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'signedContext'
  | 'timestamp'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp'
  | 'vaultInputsContext'
  | 'vaultOutputsContext';

export type Erc20 = {
  __typename?: 'ERC20';
  decimals: Scalars['Int'];
  id: Scalars['ID'];
  name: Scalars['String'];
  symbol: Scalars['String'];
  totalSupply: Scalars['BigInt'];
  totalSupplyDisplay: Scalars['BigDecimal'];
};

export type Erc20_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Erc20_Filter>>>;
  decimals?: InputMaybe<Scalars['Int']>;
  decimals_gt?: InputMaybe<Scalars['Int']>;
  decimals_gte?: InputMaybe<Scalars['Int']>;
  decimals_in?: InputMaybe<Array<Scalars['Int']>>;
  decimals_lt?: InputMaybe<Scalars['Int']>;
  decimals_lte?: InputMaybe<Scalars['Int']>;
  decimals_not?: InputMaybe<Scalars['Int']>;
  decimals_not_in?: InputMaybe<Array<Scalars['Int']>>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  name?: InputMaybe<Scalars['String']>;
  name_contains?: InputMaybe<Scalars['String']>;
  name_contains_nocase?: InputMaybe<Scalars['String']>;
  name_ends_with?: InputMaybe<Scalars['String']>;
  name_ends_with_nocase?: InputMaybe<Scalars['String']>;
  name_gt?: InputMaybe<Scalars['String']>;
  name_gte?: InputMaybe<Scalars['String']>;
  name_in?: InputMaybe<Array<Scalars['String']>>;
  name_lt?: InputMaybe<Scalars['String']>;
  name_lte?: InputMaybe<Scalars['String']>;
  name_not?: InputMaybe<Scalars['String']>;
  name_not_contains?: InputMaybe<Scalars['String']>;
  name_not_contains_nocase?: InputMaybe<Scalars['String']>;
  name_not_ends_with?: InputMaybe<Scalars['String']>;
  name_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  name_not_in?: InputMaybe<Array<Scalars['String']>>;
  name_not_starts_with?: InputMaybe<Scalars['String']>;
  name_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  name_starts_with?: InputMaybe<Scalars['String']>;
  name_starts_with_nocase?: InputMaybe<Scalars['String']>;
  or?: InputMaybe<Array<InputMaybe<Erc20_Filter>>>;
  symbol?: InputMaybe<Scalars['String']>;
  symbol_contains?: InputMaybe<Scalars['String']>;
  symbol_contains_nocase?: InputMaybe<Scalars['String']>;
  symbol_ends_with?: InputMaybe<Scalars['String']>;
  symbol_ends_with_nocase?: InputMaybe<Scalars['String']>;
  symbol_gt?: InputMaybe<Scalars['String']>;
  symbol_gte?: InputMaybe<Scalars['String']>;
  symbol_in?: InputMaybe<Array<Scalars['String']>>;
  symbol_lt?: InputMaybe<Scalars['String']>;
  symbol_lte?: InputMaybe<Scalars['String']>;
  symbol_not?: InputMaybe<Scalars['String']>;
  symbol_not_contains?: InputMaybe<Scalars['String']>;
  symbol_not_contains_nocase?: InputMaybe<Scalars['String']>;
  symbol_not_ends_with?: InputMaybe<Scalars['String']>;
  symbol_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  symbol_not_in?: InputMaybe<Array<Scalars['String']>>;
  symbol_not_starts_with?: InputMaybe<Scalars['String']>;
  symbol_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  symbol_starts_with?: InputMaybe<Scalars['String']>;
  symbol_starts_with_nocase?: InputMaybe<Scalars['String']>;
  totalSupply?: InputMaybe<Scalars['BigInt']>;
  totalSupplyDisplay?: InputMaybe<Scalars['BigDecimal']>;
  totalSupplyDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  totalSupplyDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  totalSupplyDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  totalSupplyDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  totalSupplyDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  totalSupplyDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  totalSupplyDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  totalSupply_gt?: InputMaybe<Scalars['BigInt']>;
  totalSupply_gte?: InputMaybe<Scalars['BigInt']>;
  totalSupply_in?: InputMaybe<Array<Scalars['BigInt']>>;
  totalSupply_lt?: InputMaybe<Scalars['BigInt']>;
  totalSupply_lte?: InputMaybe<Scalars['BigInt']>;
  totalSupply_not?: InputMaybe<Scalars['BigInt']>;
  totalSupply_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
};

export type Erc20_OrderBy =
  | 'decimals'
  | 'id'
  | 'name'
  | 'symbol'
  | 'totalSupply'
  | 'totalSupplyDisplay';

export type Event = {
  /** Account that sent the transaction this event was emitted in. */
  emitter: Account;
  id: Scalars['ID'];
  timestamp: Scalars['BigInt'];
  /** Transaction this event was emitted in. */
  transaction: Transaction;
};

export type Event_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Event_Filter>>>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<Event_Filter>>>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type Event_OrderBy =
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'timestamp'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp';

export type Io = {
  __typename?: 'IO';
  decimals: Scalars['Int'];
  id: Scalars['ID'];
  index: Scalars['Int'];
  order: Order;
  token: Erc20;
  tokenVault: TokenVault;
  vault: Vault;
  vaultId: Scalars['BigInt'];
};

export type Io_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Io_Filter>>>;
  decimals?: InputMaybe<Scalars['Int']>;
  decimals_gt?: InputMaybe<Scalars['Int']>;
  decimals_gte?: InputMaybe<Scalars['Int']>;
  decimals_in?: InputMaybe<Array<Scalars['Int']>>;
  decimals_lt?: InputMaybe<Scalars['Int']>;
  decimals_lte?: InputMaybe<Scalars['Int']>;
  decimals_not?: InputMaybe<Scalars['Int']>;
  decimals_not_in?: InputMaybe<Array<Scalars['Int']>>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  index?: InputMaybe<Scalars['Int']>;
  index_gt?: InputMaybe<Scalars['Int']>;
  index_gte?: InputMaybe<Scalars['Int']>;
  index_in?: InputMaybe<Array<Scalars['Int']>>;
  index_lt?: InputMaybe<Scalars['Int']>;
  index_lte?: InputMaybe<Scalars['Int']>;
  index_not?: InputMaybe<Scalars['Int']>;
  index_not_in?: InputMaybe<Array<Scalars['Int']>>;
  or?: InputMaybe<Array<InputMaybe<Io_Filter>>>;
  order?: InputMaybe<Scalars['String']>;
  order_?: InputMaybe<Order_Filter>;
  order_contains?: InputMaybe<Scalars['String']>;
  order_contains_nocase?: InputMaybe<Scalars['String']>;
  order_ends_with?: InputMaybe<Scalars['String']>;
  order_ends_with_nocase?: InputMaybe<Scalars['String']>;
  order_gt?: InputMaybe<Scalars['String']>;
  order_gte?: InputMaybe<Scalars['String']>;
  order_in?: InputMaybe<Array<Scalars['String']>>;
  order_lt?: InputMaybe<Scalars['String']>;
  order_lte?: InputMaybe<Scalars['String']>;
  order_not?: InputMaybe<Scalars['String']>;
  order_not_contains?: InputMaybe<Scalars['String']>;
  order_not_contains_nocase?: InputMaybe<Scalars['String']>;
  order_not_ends_with?: InputMaybe<Scalars['String']>;
  order_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  order_not_in?: InputMaybe<Array<Scalars['String']>>;
  order_not_starts_with?: InputMaybe<Scalars['String']>;
  order_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  order_starts_with?: InputMaybe<Scalars['String']>;
  order_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token?: InputMaybe<Scalars['String']>;
  tokenVault?: InputMaybe<Scalars['String']>;
  tokenVault_?: InputMaybe<TokenVault_Filter>;
  tokenVault_contains?: InputMaybe<Scalars['String']>;
  tokenVault_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_ends_with?: InputMaybe<Scalars['String']>;
  tokenVault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_gt?: InputMaybe<Scalars['String']>;
  tokenVault_gte?: InputMaybe<Scalars['String']>;
  tokenVault_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVault_lt?: InputMaybe<Scalars['String']>;
  tokenVault_lte?: InputMaybe<Scalars['String']>;
  tokenVault_not?: InputMaybe<Scalars['String']>;
  tokenVault_not_contains?: InputMaybe<Scalars['String']>;
  tokenVault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_not_ends_with?: InputMaybe<Scalars['String']>;
  tokenVault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_not_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVault_not_starts_with?: InputMaybe<Scalars['String']>;
  tokenVault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_starts_with?: InputMaybe<Scalars['String']>;
  tokenVault_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_?: InputMaybe<Erc20_Filter>;
  token_contains?: InputMaybe<Scalars['String']>;
  token_contains_nocase?: InputMaybe<Scalars['String']>;
  token_ends_with?: InputMaybe<Scalars['String']>;
  token_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_gt?: InputMaybe<Scalars['String']>;
  token_gte?: InputMaybe<Scalars['String']>;
  token_in?: InputMaybe<Array<Scalars['String']>>;
  token_lt?: InputMaybe<Scalars['String']>;
  token_lte?: InputMaybe<Scalars['String']>;
  token_not?: InputMaybe<Scalars['String']>;
  token_not_contains?: InputMaybe<Scalars['String']>;
  token_not_contains_nocase?: InputMaybe<Scalars['String']>;
  token_not_ends_with?: InputMaybe<Scalars['String']>;
  token_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_not_in?: InputMaybe<Array<Scalars['String']>>;
  token_not_starts_with?: InputMaybe<Scalars['String']>;
  token_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_starts_with?: InputMaybe<Scalars['String']>;
  token_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault?: InputMaybe<Scalars['String']>;
  vaultId?: InputMaybe<Scalars['BigInt']>;
  vaultId_gt?: InputMaybe<Scalars['BigInt']>;
  vaultId_gte?: InputMaybe<Scalars['BigInt']>;
  vaultId_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultId_lt?: InputMaybe<Scalars['BigInt']>;
  vaultId_lte?: InputMaybe<Scalars['BigInt']>;
  vaultId_not?: InputMaybe<Scalars['BigInt']>;
  vaultId_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vault_?: InputMaybe<Vault_Filter>;
  vault_contains?: InputMaybe<Scalars['String']>;
  vault_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_ends_with?: InputMaybe<Scalars['String']>;
  vault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_gt?: InputMaybe<Scalars['String']>;
  vault_gte?: InputMaybe<Scalars['String']>;
  vault_in?: InputMaybe<Array<Scalars['String']>>;
  vault_lt?: InputMaybe<Scalars['String']>;
  vault_lte?: InputMaybe<Scalars['String']>;
  vault_not?: InputMaybe<Scalars['String']>;
  vault_not_contains?: InputMaybe<Scalars['String']>;
  vault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_not_ends_with?: InputMaybe<Scalars['String']>;
  vault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_not_in?: InputMaybe<Array<Scalars['String']>>;
  vault_not_starts_with?: InputMaybe<Scalars['String']>;
  vault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault_starts_with?: InputMaybe<Scalars['String']>;
  vault_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type Io_OrderBy =
  | 'decimals'
  | 'id'
  | 'index'
  | 'order'
  | 'order__expression'
  | 'order__expressionDeployer'
  | 'order__handleIO'
  | 'order__id'
  | 'order__interpreter'
  | 'order__interpreterStore'
  | 'order__orderActive'
  | 'order__orderHash'
  | 'order__orderJSONString'
  | 'order__timestamp'
  | 'token'
  | 'tokenVault'
  | 'tokenVault__balance'
  | 'tokenVault__balanceDisplay'
  | 'tokenVault__id'
  | 'tokenVault__vaultId'
  | 'token__decimals'
  | 'token__id'
  | 'token__name'
  | 'token__symbol'
  | 'token__totalSupply'
  | 'token__totalSupplyDisplay'
  | 'vault'
  | 'vaultId'
  | 'vault__id'
  | 'vault__vaultId';

export type MetaContentV1 = {
  __typename?: 'MetaContentV1';
  /** The header name info for Content-Encoding. It's optional */
  contentEncoding?: Maybe<Scalars['String']>;
  /** The header name info for Content-Language. It's optional */
  contentLanguage?: Maybe<Scalars['String']>;
  /** The header name info for Content-Type */
  contentType?: Maybe<Scalars['String']>;
  /** RainMeta documents bytes that have this content */
  documents: Array<RainMetaV1>;
  /** The hash of the Map Rain Meta document or CBOR Item */
  id: Scalars['Bytes'];
  /** The magic number that is used to track the payload */
  magicNumber: Scalars['BigInt'];
  /** The payload present on the index 0 of the Rain meta Document */
  payload: Scalars['Bytes'];
};


export type MetaContentV1DocumentsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<RainMetaV1_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<RainMetaV1_Filter>;
};

export type MetaContentV1_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<MetaContentV1_Filter>>>;
  contentEncoding?: InputMaybe<Scalars['String']>;
  contentEncoding_contains?: InputMaybe<Scalars['String']>;
  contentEncoding_contains_nocase?: InputMaybe<Scalars['String']>;
  contentEncoding_ends_with?: InputMaybe<Scalars['String']>;
  contentEncoding_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contentEncoding_gt?: InputMaybe<Scalars['String']>;
  contentEncoding_gte?: InputMaybe<Scalars['String']>;
  contentEncoding_in?: InputMaybe<Array<Scalars['String']>>;
  contentEncoding_lt?: InputMaybe<Scalars['String']>;
  contentEncoding_lte?: InputMaybe<Scalars['String']>;
  contentEncoding_not?: InputMaybe<Scalars['String']>;
  contentEncoding_not_contains?: InputMaybe<Scalars['String']>;
  contentEncoding_not_contains_nocase?: InputMaybe<Scalars['String']>;
  contentEncoding_not_ends_with?: InputMaybe<Scalars['String']>;
  contentEncoding_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contentEncoding_not_in?: InputMaybe<Array<Scalars['String']>>;
  contentEncoding_not_starts_with?: InputMaybe<Scalars['String']>;
  contentEncoding_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  contentEncoding_starts_with?: InputMaybe<Scalars['String']>;
  contentEncoding_starts_with_nocase?: InputMaybe<Scalars['String']>;
  contentLanguage?: InputMaybe<Scalars['String']>;
  contentLanguage_contains?: InputMaybe<Scalars['String']>;
  contentLanguage_contains_nocase?: InputMaybe<Scalars['String']>;
  contentLanguage_ends_with?: InputMaybe<Scalars['String']>;
  contentLanguage_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contentLanguage_gt?: InputMaybe<Scalars['String']>;
  contentLanguage_gte?: InputMaybe<Scalars['String']>;
  contentLanguage_in?: InputMaybe<Array<Scalars['String']>>;
  contentLanguage_lt?: InputMaybe<Scalars['String']>;
  contentLanguage_lte?: InputMaybe<Scalars['String']>;
  contentLanguage_not?: InputMaybe<Scalars['String']>;
  contentLanguage_not_contains?: InputMaybe<Scalars['String']>;
  contentLanguage_not_contains_nocase?: InputMaybe<Scalars['String']>;
  contentLanguage_not_ends_with?: InputMaybe<Scalars['String']>;
  contentLanguage_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contentLanguage_not_in?: InputMaybe<Array<Scalars['String']>>;
  contentLanguage_not_starts_with?: InputMaybe<Scalars['String']>;
  contentLanguage_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  contentLanguage_starts_with?: InputMaybe<Scalars['String']>;
  contentLanguage_starts_with_nocase?: InputMaybe<Scalars['String']>;
  contentType?: InputMaybe<Scalars['String']>;
  contentType_contains?: InputMaybe<Scalars['String']>;
  contentType_contains_nocase?: InputMaybe<Scalars['String']>;
  contentType_ends_with?: InputMaybe<Scalars['String']>;
  contentType_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contentType_gt?: InputMaybe<Scalars['String']>;
  contentType_gte?: InputMaybe<Scalars['String']>;
  contentType_in?: InputMaybe<Array<Scalars['String']>>;
  contentType_lt?: InputMaybe<Scalars['String']>;
  contentType_lte?: InputMaybe<Scalars['String']>;
  contentType_not?: InputMaybe<Scalars['String']>;
  contentType_not_contains?: InputMaybe<Scalars['String']>;
  contentType_not_contains_nocase?: InputMaybe<Scalars['String']>;
  contentType_not_ends_with?: InputMaybe<Scalars['String']>;
  contentType_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  contentType_not_in?: InputMaybe<Array<Scalars['String']>>;
  contentType_not_starts_with?: InputMaybe<Scalars['String']>;
  contentType_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  contentType_starts_with?: InputMaybe<Scalars['String']>;
  contentType_starts_with_nocase?: InputMaybe<Scalars['String']>;
  documents?: InputMaybe<Array<Scalars['String']>>;
  documents_?: InputMaybe<RainMetaV1_Filter>;
  documents_contains?: InputMaybe<Array<Scalars['String']>>;
  documents_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  documents_not?: InputMaybe<Array<Scalars['String']>>;
  documents_not_contains?: InputMaybe<Array<Scalars['String']>>;
  documents_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  id?: InputMaybe<Scalars['Bytes']>;
  id_contains?: InputMaybe<Scalars['Bytes']>;
  id_gt?: InputMaybe<Scalars['Bytes']>;
  id_gte?: InputMaybe<Scalars['Bytes']>;
  id_in?: InputMaybe<Array<Scalars['Bytes']>>;
  id_lt?: InputMaybe<Scalars['Bytes']>;
  id_lte?: InputMaybe<Scalars['Bytes']>;
  id_not?: InputMaybe<Scalars['Bytes']>;
  id_not_contains?: InputMaybe<Scalars['Bytes']>;
  id_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  magicNumber?: InputMaybe<Scalars['BigInt']>;
  magicNumber_gt?: InputMaybe<Scalars['BigInt']>;
  magicNumber_gte?: InputMaybe<Scalars['BigInt']>;
  magicNumber_in?: InputMaybe<Array<Scalars['BigInt']>>;
  magicNumber_lt?: InputMaybe<Scalars['BigInt']>;
  magicNumber_lte?: InputMaybe<Scalars['BigInt']>;
  magicNumber_not?: InputMaybe<Scalars['BigInt']>;
  magicNumber_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  or?: InputMaybe<Array<InputMaybe<MetaContentV1_Filter>>>;
  payload?: InputMaybe<Scalars['Bytes']>;
  payload_contains?: InputMaybe<Scalars['Bytes']>;
  payload_gt?: InputMaybe<Scalars['Bytes']>;
  payload_gte?: InputMaybe<Scalars['Bytes']>;
  payload_in?: InputMaybe<Array<Scalars['Bytes']>>;
  payload_lt?: InputMaybe<Scalars['Bytes']>;
  payload_lte?: InputMaybe<Scalars['Bytes']>;
  payload_not?: InputMaybe<Scalars['Bytes']>;
  payload_not_contains?: InputMaybe<Scalars['Bytes']>;
  payload_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
};

export type MetaContentV1_OrderBy =
  | 'contentEncoding'
  | 'contentLanguage'
  | 'contentType'
  | 'documents'
  | 'id'
  | 'magicNumber'
  | 'payload';

export type Order = Event & {
  __typename?: 'Order';
  emitter: Account;
  /** The address to the rain expression for the Order */
  expression: Scalars['Bytes'];
  /** The IExpressionDeployer contract address that is used to add the order */
  expressionDeployer: Scalars['Bytes'];
  /** Flag that check if there is a handle_IO entrypoint to run. If false the order book MAY skip calling the interpreter to save gas */
  handleIO: Scalars['Boolean'];
  /** The hash of the order */
  id: Scalars['ID'];
  /** The IInterpreter address that is used to add the order */
  interpreter: Scalars['Bytes'];
  /** The IInterpreterStore address that is used to add the order */
  interpreterStore: Scalars['Bytes'];
  meta?: Maybe<RainMetaV1>;
  /** Whether the order is active or inactive */
  orderActive: Scalars['Boolean'];
  /** The hash of the order */
  orderHash: Scalars['Bytes'];
  orderJSONString: Scalars['String'];
  /** Order Clear entities that use this order */
  ordersClears?: Maybe<Array<OrderClear>>;
  /** The address that added the order */
  owner: Account;
  /** Take Order entities that use this order */
  takeOrders?: Maybe<Array<TakeOrderEntity>>;
  timestamp: Scalars['BigInt'];
  /** Timestamp when the order was added */
  transaction: Transaction;
  /** validInputs */
  validInputs?: Maybe<Array<Io>>;
  /** validOutputs */
  validOutputs?: Maybe<Array<Io>>;
};


export type OrderOrdersClearsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClear_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<OrderClear_Filter>;
};


export type OrderTakeOrdersArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TakeOrderEntity_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<TakeOrderEntity_Filter>;
};


export type OrderValidInputsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Io_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Io_Filter>;
};


export type OrderValidOutputsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Io_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Io_Filter>;
};

export type OrderBook = {
  __typename?: 'OrderBook';
  address: Scalars['Bytes'];
  deployer?: Maybe<Scalars['Bytes']>;
  id: Scalars['Bytes'];
  /** The RainMetaV1 decode information */
  meta?: Maybe<RainMetaV1>;
};

export type OrderBook_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  address?: InputMaybe<Scalars['Bytes']>;
  address_contains?: InputMaybe<Scalars['Bytes']>;
  address_gt?: InputMaybe<Scalars['Bytes']>;
  address_gte?: InputMaybe<Scalars['Bytes']>;
  address_in?: InputMaybe<Array<Scalars['Bytes']>>;
  address_lt?: InputMaybe<Scalars['Bytes']>;
  address_lte?: InputMaybe<Scalars['Bytes']>;
  address_not?: InputMaybe<Scalars['Bytes']>;
  address_not_contains?: InputMaybe<Scalars['Bytes']>;
  address_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  and?: InputMaybe<Array<InputMaybe<OrderBook_Filter>>>;
  deployer?: InputMaybe<Scalars['Bytes']>;
  deployer_contains?: InputMaybe<Scalars['Bytes']>;
  deployer_gt?: InputMaybe<Scalars['Bytes']>;
  deployer_gte?: InputMaybe<Scalars['Bytes']>;
  deployer_in?: InputMaybe<Array<Scalars['Bytes']>>;
  deployer_lt?: InputMaybe<Scalars['Bytes']>;
  deployer_lte?: InputMaybe<Scalars['Bytes']>;
  deployer_not?: InputMaybe<Scalars['Bytes']>;
  deployer_not_contains?: InputMaybe<Scalars['Bytes']>;
  deployer_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  id?: InputMaybe<Scalars['Bytes']>;
  id_contains?: InputMaybe<Scalars['Bytes']>;
  id_gt?: InputMaybe<Scalars['Bytes']>;
  id_gte?: InputMaybe<Scalars['Bytes']>;
  id_in?: InputMaybe<Array<Scalars['Bytes']>>;
  id_lt?: InputMaybe<Scalars['Bytes']>;
  id_lte?: InputMaybe<Scalars['Bytes']>;
  id_not?: InputMaybe<Scalars['Bytes']>;
  id_not_contains?: InputMaybe<Scalars['Bytes']>;
  id_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  meta?: InputMaybe<Scalars['String']>;
  meta_?: InputMaybe<RainMetaV1_Filter>;
  meta_contains?: InputMaybe<Scalars['String']>;
  meta_contains_nocase?: InputMaybe<Scalars['String']>;
  meta_ends_with?: InputMaybe<Scalars['String']>;
  meta_ends_with_nocase?: InputMaybe<Scalars['String']>;
  meta_gt?: InputMaybe<Scalars['String']>;
  meta_gte?: InputMaybe<Scalars['String']>;
  meta_in?: InputMaybe<Array<Scalars['String']>>;
  meta_lt?: InputMaybe<Scalars['String']>;
  meta_lte?: InputMaybe<Scalars['String']>;
  meta_not?: InputMaybe<Scalars['String']>;
  meta_not_contains?: InputMaybe<Scalars['String']>;
  meta_not_contains_nocase?: InputMaybe<Scalars['String']>;
  meta_not_ends_with?: InputMaybe<Scalars['String']>;
  meta_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  meta_not_in?: InputMaybe<Array<Scalars['String']>>;
  meta_not_starts_with?: InputMaybe<Scalars['String']>;
  meta_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  meta_starts_with?: InputMaybe<Scalars['String']>;
  meta_starts_with_nocase?: InputMaybe<Scalars['String']>;
  or?: InputMaybe<Array<InputMaybe<OrderBook_Filter>>>;
};

export type OrderBook_OrderBy =
  | 'address'
  | 'deployer'
  | 'id'
  | 'meta'
  | 'meta__id'
  | 'meta__metaBytes';

export type OrderClear = Event & {
  __typename?: 'OrderClear';
  /** The token input index cleared into Order A */
  aInputIOIndex: Scalars['BigInt'];
  /** The token output index cleared into Order A */
  aOutputIOIndex: Scalars['BigInt'];
  /** The token input index cleared into Order B */
  bInputIOIndex: Scalars['BigInt'];
  /** The token output index cleared into Order B */
  bOutputIOIndex: Scalars['BigInt'];
  /** The bounty paid when this order was cleared */
  bounty: Bounty;
  /** The clearer address who cleared this order */
  clearer: Account;
  emitter: Account;
  id: Scalars['ID'];
  /** Order A being cleared */
  orderA: Order;
  /** Order B being cleared */
  orderB: Order;
  /** The owners of the Orders that were cleared [Order A, Order B] */
  owners?: Maybe<Array<Account>>;
  /** The sender address who cleared the Orders */
  sender: Account;
  /** The state change that occurred because of this Clear */
  stateChange: OrderClearStateChange;
  timestamp: Scalars['BigInt'];
  transaction: Transaction;
};


export type OrderClearOwnersArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Account_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Account_Filter>;
};

export type OrderClearStateChange = {
  __typename?: 'OrderClearStateChange';
  aInput: Scalars['BigInt'];
  aOutput: Scalars['BigInt'];
  bInput: Scalars['BigInt'];
  bOutput: Scalars['BigInt'];
  id: Scalars['ID'];
  orderClear: OrderClear;
};

export type OrderClearStateChange_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  aInput?: InputMaybe<Scalars['BigInt']>;
  aInput_gt?: InputMaybe<Scalars['BigInt']>;
  aInput_gte?: InputMaybe<Scalars['BigInt']>;
  aInput_in?: InputMaybe<Array<Scalars['BigInt']>>;
  aInput_lt?: InputMaybe<Scalars['BigInt']>;
  aInput_lte?: InputMaybe<Scalars['BigInt']>;
  aInput_not?: InputMaybe<Scalars['BigInt']>;
  aInput_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  aOutput?: InputMaybe<Scalars['BigInt']>;
  aOutput_gt?: InputMaybe<Scalars['BigInt']>;
  aOutput_gte?: InputMaybe<Scalars['BigInt']>;
  aOutput_in?: InputMaybe<Array<Scalars['BigInt']>>;
  aOutput_lt?: InputMaybe<Scalars['BigInt']>;
  aOutput_lte?: InputMaybe<Scalars['BigInt']>;
  aOutput_not?: InputMaybe<Scalars['BigInt']>;
  aOutput_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  and?: InputMaybe<Array<InputMaybe<OrderClearStateChange_Filter>>>;
  bInput?: InputMaybe<Scalars['BigInt']>;
  bInput_gt?: InputMaybe<Scalars['BigInt']>;
  bInput_gte?: InputMaybe<Scalars['BigInt']>;
  bInput_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bInput_lt?: InputMaybe<Scalars['BigInt']>;
  bInput_lte?: InputMaybe<Scalars['BigInt']>;
  bInput_not?: InputMaybe<Scalars['BigInt']>;
  bInput_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bOutput?: InputMaybe<Scalars['BigInt']>;
  bOutput_gt?: InputMaybe<Scalars['BigInt']>;
  bOutput_gte?: InputMaybe<Scalars['BigInt']>;
  bOutput_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bOutput_lt?: InputMaybe<Scalars['BigInt']>;
  bOutput_lte?: InputMaybe<Scalars['BigInt']>;
  bOutput_not?: InputMaybe<Scalars['BigInt']>;
  bOutput_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<OrderClearStateChange_Filter>>>;
  orderClear?: InputMaybe<Scalars['String']>;
  orderClear_?: InputMaybe<OrderClear_Filter>;
  orderClear_contains?: InputMaybe<Scalars['String']>;
  orderClear_contains_nocase?: InputMaybe<Scalars['String']>;
  orderClear_ends_with?: InputMaybe<Scalars['String']>;
  orderClear_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderClear_gt?: InputMaybe<Scalars['String']>;
  orderClear_gte?: InputMaybe<Scalars['String']>;
  orderClear_in?: InputMaybe<Array<Scalars['String']>>;
  orderClear_lt?: InputMaybe<Scalars['String']>;
  orderClear_lte?: InputMaybe<Scalars['String']>;
  orderClear_not?: InputMaybe<Scalars['String']>;
  orderClear_not_contains?: InputMaybe<Scalars['String']>;
  orderClear_not_contains_nocase?: InputMaybe<Scalars['String']>;
  orderClear_not_ends_with?: InputMaybe<Scalars['String']>;
  orderClear_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderClear_not_in?: InputMaybe<Array<Scalars['String']>>;
  orderClear_not_starts_with?: InputMaybe<Scalars['String']>;
  orderClear_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderClear_starts_with?: InputMaybe<Scalars['String']>;
  orderClear_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type OrderClearStateChange_OrderBy =
  | 'aInput'
  | 'aOutput'
  | 'bInput'
  | 'bOutput'
  | 'id'
  | 'orderClear'
  | 'orderClear__aInputIOIndex'
  | 'orderClear__aOutputIOIndex'
  | 'orderClear__bInputIOIndex'
  | 'orderClear__bOutputIOIndex'
  | 'orderClear__id'
  | 'orderClear__timestamp';

export type OrderClear_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  aInputIOIndex?: InputMaybe<Scalars['BigInt']>;
  aInputIOIndex_gt?: InputMaybe<Scalars['BigInt']>;
  aInputIOIndex_gte?: InputMaybe<Scalars['BigInt']>;
  aInputIOIndex_in?: InputMaybe<Array<Scalars['BigInt']>>;
  aInputIOIndex_lt?: InputMaybe<Scalars['BigInt']>;
  aInputIOIndex_lte?: InputMaybe<Scalars['BigInt']>;
  aInputIOIndex_not?: InputMaybe<Scalars['BigInt']>;
  aInputIOIndex_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  aOutputIOIndex?: InputMaybe<Scalars['BigInt']>;
  aOutputIOIndex_gt?: InputMaybe<Scalars['BigInt']>;
  aOutputIOIndex_gte?: InputMaybe<Scalars['BigInt']>;
  aOutputIOIndex_in?: InputMaybe<Array<Scalars['BigInt']>>;
  aOutputIOIndex_lt?: InputMaybe<Scalars['BigInt']>;
  aOutputIOIndex_lte?: InputMaybe<Scalars['BigInt']>;
  aOutputIOIndex_not?: InputMaybe<Scalars['BigInt']>;
  aOutputIOIndex_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  and?: InputMaybe<Array<InputMaybe<OrderClear_Filter>>>;
  bInputIOIndex?: InputMaybe<Scalars['BigInt']>;
  bInputIOIndex_gt?: InputMaybe<Scalars['BigInt']>;
  bInputIOIndex_gte?: InputMaybe<Scalars['BigInt']>;
  bInputIOIndex_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bInputIOIndex_lt?: InputMaybe<Scalars['BigInt']>;
  bInputIOIndex_lte?: InputMaybe<Scalars['BigInt']>;
  bInputIOIndex_not?: InputMaybe<Scalars['BigInt']>;
  bInputIOIndex_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bOutputIOIndex?: InputMaybe<Scalars['BigInt']>;
  bOutputIOIndex_gt?: InputMaybe<Scalars['BigInt']>;
  bOutputIOIndex_gte?: InputMaybe<Scalars['BigInt']>;
  bOutputIOIndex_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bOutputIOIndex_lt?: InputMaybe<Scalars['BigInt']>;
  bOutputIOIndex_lte?: InputMaybe<Scalars['BigInt']>;
  bOutputIOIndex_not?: InputMaybe<Scalars['BigInt']>;
  bOutputIOIndex_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  bounty_?: InputMaybe<Bounty_Filter>;
  clearer?: InputMaybe<Scalars['String']>;
  clearer_?: InputMaybe<Account_Filter>;
  clearer_contains?: InputMaybe<Scalars['String']>;
  clearer_contains_nocase?: InputMaybe<Scalars['String']>;
  clearer_ends_with?: InputMaybe<Scalars['String']>;
  clearer_ends_with_nocase?: InputMaybe<Scalars['String']>;
  clearer_gt?: InputMaybe<Scalars['String']>;
  clearer_gte?: InputMaybe<Scalars['String']>;
  clearer_in?: InputMaybe<Array<Scalars['String']>>;
  clearer_lt?: InputMaybe<Scalars['String']>;
  clearer_lte?: InputMaybe<Scalars['String']>;
  clearer_not?: InputMaybe<Scalars['String']>;
  clearer_not_contains?: InputMaybe<Scalars['String']>;
  clearer_not_contains_nocase?: InputMaybe<Scalars['String']>;
  clearer_not_ends_with?: InputMaybe<Scalars['String']>;
  clearer_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  clearer_not_in?: InputMaybe<Array<Scalars['String']>>;
  clearer_not_starts_with?: InputMaybe<Scalars['String']>;
  clearer_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  clearer_starts_with?: InputMaybe<Scalars['String']>;
  clearer_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<OrderClear_Filter>>>;
  orderA?: InputMaybe<Scalars['String']>;
  orderA_?: InputMaybe<Order_Filter>;
  orderA_contains?: InputMaybe<Scalars['String']>;
  orderA_contains_nocase?: InputMaybe<Scalars['String']>;
  orderA_ends_with?: InputMaybe<Scalars['String']>;
  orderA_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderA_gt?: InputMaybe<Scalars['String']>;
  orderA_gte?: InputMaybe<Scalars['String']>;
  orderA_in?: InputMaybe<Array<Scalars['String']>>;
  orderA_lt?: InputMaybe<Scalars['String']>;
  orderA_lte?: InputMaybe<Scalars['String']>;
  orderA_not?: InputMaybe<Scalars['String']>;
  orderA_not_contains?: InputMaybe<Scalars['String']>;
  orderA_not_contains_nocase?: InputMaybe<Scalars['String']>;
  orderA_not_ends_with?: InputMaybe<Scalars['String']>;
  orderA_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderA_not_in?: InputMaybe<Array<Scalars['String']>>;
  orderA_not_starts_with?: InputMaybe<Scalars['String']>;
  orderA_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderA_starts_with?: InputMaybe<Scalars['String']>;
  orderA_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderB?: InputMaybe<Scalars['String']>;
  orderB_?: InputMaybe<Order_Filter>;
  orderB_contains?: InputMaybe<Scalars['String']>;
  orderB_contains_nocase?: InputMaybe<Scalars['String']>;
  orderB_ends_with?: InputMaybe<Scalars['String']>;
  orderB_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderB_gt?: InputMaybe<Scalars['String']>;
  orderB_gte?: InputMaybe<Scalars['String']>;
  orderB_in?: InputMaybe<Array<Scalars['String']>>;
  orderB_lt?: InputMaybe<Scalars['String']>;
  orderB_lte?: InputMaybe<Scalars['String']>;
  orderB_not?: InputMaybe<Scalars['String']>;
  orderB_not_contains?: InputMaybe<Scalars['String']>;
  orderB_not_contains_nocase?: InputMaybe<Scalars['String']>;
  orderB_not_ends_with?: InputMaybe<Scalars['String']>;
  orderB_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderB_not_in?: InputMaybe<Array<Scalars['String']>>;
  orderB_not_starts_with?: InputMaybe<Scalars['String']>;
  orderB_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderB_starts_with?: InputMaybe<Scalars['String']>;
  orderB_starts_with_nocase?: InputMaybe<Scalars['String']>;
  owners?: InputMaybe<Array<Scalars['String']>>;
  owners_?: InputMaybe<Account_Filter>;
  owners_contains?: InputMaybe<Array<Scalars['String']>>;
  owners_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  owners_not?: InputMaybe<Array<Scalars['String']>>;
  owners_not_contains?: InputMaybe<Array<Scalars['String']>>;
  owners_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  sender?: InputMaybe<Scalars['String']>;
  sender_?: InputMaybe<Account_Filter>;
  sender_contains?: InputMaybe<Scalars['String']>;
  sender_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_ends_with?: InputMaybe<Scalars['String']>;
  sender_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_gt?: InputMaybe<Scalars['String']>;
  sender_gte?: InputMaybe<Scalars['String']>;
  sender_in?: InputMaybe<Array<Scalars['String']>>;
  sender_lt?: InputMaybe<Scalars['String']>;
  sender_lte?: InputMaybe<Scalars['String']>;
  sender_not?: InputMaybe<Scalars['String']>;
  sender_not_contains?: InputMaybe<Scalars['String']>;
  sender_not_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_not_ends_with?: InputMaybe<Scalars['String']>;
  sender_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_not_in?: InputMaybe<Array<Scalars['String']>>;
  sender_not_starts_with?: InputMaybe<Scalars['String']>;
  sender_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  sender_starts_with?: InputMaybe<Scalars['String']>;
  sender_starts_with_nocase?: InputMaybe<Scalars['String']>;
  stateChange_?: InputMaybe<OrderClearStateChange_Filter>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type OrderClear_OrderBy =
  | 'aInputIOIndex'
  | 'aOutputIOIndex'
  | 'bInputIOIndex'
  | 'bOutputIOIndex'
  | 'bounty'
  | 'bounty__bountyAmountA'
  | 'bounty__bountyAmountADisplay'
  | 'bounty__bountyAmountB'
  | 'bounty__bountyAmountBDisplay'
  | 'bounty__id'
  | 'bounty__timestamp'
  | 'clearer'
  | 'clearer__id'
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'orderA'
  | 'orderA__expression'
  | 'orderA__expressionDeployer'
  | 'orderA__handleIO'
  | 'orderA__id'
  | 'orderA__interpreter'
  | 'orderA__interpreterStore'
  | 'orderA__orderActive'
  | 'orderA__orderHash'
  | 'orderA__orderJSONString'
  | 'orderA__timestamp'
  | 'orderB'
  | 'orderB__expression'
  | 'orderB__expressionDeployer'
  | 'orderB__handleIO'
  | 'orderB__id'
  | 'orderB__interpreter'
  | 'orderB__interpreterStore'
  | 'orderB__orderActive'
  | 'orderB__orderHash'
  | 'orderB__orderJSONString'
  | 'orderB__timestamp'
  | 'owners'
  | 'sender'
  | 'sender__id'
  | 'stateChange'
  | 'stateChange__aInput'
  | 'stateChange__aOutput'
  | 'stateChange__bInput'
  | 'stateChange__bOutput'
  | 'stateChange__id'
  | 'timestamp'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp';

/** Defines the order direction, either ascending or descending */
export type OrderDirection =
  | 'asc'
  | 'desc';

export type Order_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Order_Filter>>>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  expression?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_contains?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_gt?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_gte?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_in?: InputMaybe<Array<Scalars['Bytes']>>;
  expressionDeployer_lt?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_lte?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_not?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_not_contains?: InputMaybe<Scalars['Bytes']>;
  expressionDeployer_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  expression_contains?: InputMaybe<Scalars['Bytes']>;
  expression_gt?: InputMaybe<Scalars['Bytes']>;
  expression_gte?: InputMaybe<Scalars['Bytes']>;
  expression_in?: InputMaybe<Array<Scalars['Bytes']>>;
  expression_lt?: InputMaybe<Scalars['Bytes']>;
  expression_lte?: InputMaybe<Scalars['Bytes']>;
  expression_not?: InputMaybe<Scalars['Bytes']>;
  expression_not_contains?: InputMaybe<Scalars['Bytes']>;
  expression_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  handleIO?: InputMaybe<Scalars['Boolean']>;
  handleIO_in?: InputMaybe<Array<Scalars['Boolean']>>;
  handleIO_not?: InputMaybe<Scalars['Boolean']>;
  handleIO_not_in?: InputMaybe<Array<Scalars['Boolean']>>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  interpreter?: InputMaybe<Scalars['Bytes']>;
  interpreterStore?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_contains?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_gt?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_gte?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_in?: InputMaybe<Array<Scalars['Bytes']>>;
  interpreterStore_lt?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_lte?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_not?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_not_contains?: InputMaybe<Scalars['Bytes']>;
  interpreterStore_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  interpreter_contains?: InputMaybe<Scalars['Bytes']>;
  interpreter_gt?: InputMaybe<Scalars['Bytes']>;
  interpreter_gte?: InputMaybe<Scalars['Bytes']>;
  interpreter_in?: InputMaybe<Array<Scalars['Bytes']>>;
  interpreter_lt?: InputMaybe<Scalars['Bytes']>;
  interpreter_lte?: InputMaybe<Scalars['Bytes']>;
  interpreter_not?: InputMaybe<Scalars['Bytes']>;
  interpreter_not_contains?: InputMaybe<Scalars['Bytes']>;
  interpreter_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  meta?: InputMaybe<Scalars['String']>;
  meta_?: InputMaybe<RainMetaV1_Filter>;
  meta_contains?: InputMaybe<Scalars['String']>;
  meta_contains_nocase?: InputMaybe<Scalars['String']>;
  meta_ends_with?: InputMaybe<Scalars['String']>;
  meta_ends_with_nocase?: InputMaybe<Scalars['String']>;
  meta_gt?: InputMaybe<Scalars['String']>;
  meta_gte?: InputMaybe<Scalars['String']>;
  meta_in?: InputMaybe<Array<Scalars['String']>>;
  meta_lt?: InputMaybe<Scalars['String']>;
  meta_lte?: InputMaybe<Scalars['String']>;
  meta_not?: InputMaybe<Scalars['String']>;
  meta_not_contains?: InputMaybe<Scalars['String']>;
  meta_not_contains_nocase?: InputMaybe<Scalars['String']>;
  meta_not_ends_with?: InputMaybe<Scalars['String']>;
  meta_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  meta_not_in?: InputMaybe<Array<Scalars['String']>>;
  meta_not_starts_with?: InputMaybe<Scalars['String']>;
  meta_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  meta_starts_with?: InputMaybe<Scalars['String']>;
  meta_starts_with_nocase?: InputMaybe<Scalars['String']>;
  or?: InputMaybe<Array<InputMaybe<Order_Filter>>>;
  orderActive?: InputMaybe<Scalars['Boolean']>;
  orderActive_in?: InputMaybe<Array<Scalars['Boolean']>>;
  orderActive_not?: InputMaybe<Scalars['Boolean']>;
  orderActive_not_in?: InputMaybe<Array<Scalars['Boolean']>>;
  orderHash?: InputMaybe<Scalars['Bytes']>;
  orderHash_contains?: InputMaybe<Scalars['Bytes']>;
  orderHash_gt?: InputMaybe<Scalars['Bytes']>;
  orderHash_gte?: InputMaybe<Scalars['Bytes']>;
  orderHash_in?: InputMaybe<Array<Scalars['Bytes']>>;
  orderHash_lt?: InputMaybe<Scalars['Bytes']>;
  orderHash_lte?: InputMaybe<Scalars['Bytes']>;
  orderHash_not?: InputMaybe<Scalars['Bytes']>;
  orderHash_not_contains?: InputMaybe<Scalars['Bytes']>;
  orderHash_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  orderJSONString?: InputMaybe<Scalars['String']>;
  orderJSONString_contains?: InputMaybe<Scalars['String']>;
  orderJSONString_contains_nocase?: InputMaybe<Scalars['String']>;
  orderJSONString_ends_with?: InputMaybe<Scalars['String']>;
  orderJSONString_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderJSONString_gt?: InputMaybe<Scalars['String']>;
  orderJSONString_gte?: InputMaybe<Scalars['String']>;
  orderJSONString_in?: InputMaybe<Array<Scalars['String']>>;
  orderJSONString_lt?: InputMaybe<Scalars['String']>;
  orderJSONString_lte?: InputMaybe<Scalars['String']>;
  orderJSONString_not?: InputMaybe<Scalars['String']>;
  orderJSONString_not_contains?: InputMaybe<Scalars['String']>;
  orderJSONString_not_contains_nocase?: InputMaybe<Scalars['String']>;
  orderJSONString_not_ends_with?: InputMaybe<Scalars['String']>;
  orderJSONString_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  orderJSONString_not_in?: InputMaybe<Array<Scalars['String']>>;
  orderJSONString_not_starts_with?: InputMaybe<Scalars['String']>;
  orderJSONString_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  orderJSONString_starts_with?: InputMaybe<Scalars['String']>;
  orderJSONString_starts_with_nocase?: InputMaybe<Scalars['String']>;
  ordersClears?: InputMaybe<Array<Scalars['String']>>;
  ordersClears_?: InputMaybe<OrderClear_Filter>;
  ordersClears_contains?: InputMaybe<Array<Scalars['String']>>;
  ordersClears_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  ordersClears_not?: InputMaybe<Array<Scalars['String']>>;
  ordersClears_not_contains?: InputMaybe<Array<Scalars['String']>>;
  ordersClears_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  owner?: InputMaybe<Scalars['String']>;
  owner_?: InputMaybe<Account_Filter>;
  owner_contains?: InputMaybe<Scalars['String']>;
  owner_contains_nocase?: InputMaybe<Scalars['String']>;
  owner_ends_with?: InputMaybe<Scalars['String']>;
  owner_ends_with_nocase?: InputMaybe<Scalars['String']>;
  owner_gt?: InputMaybe<Scalars['String']>;
  owner_gte?: InputMaybe<Scalars['String']>;
  owner_in?: InputMaybe<Array<Scalars['String']>>;
  owner_lt?: InputMaybe<Scalars['String']>;
  owner_lte?: InputMaybe<Scalars['String']>;
  owner_not?: InputMaybe<Scalars['String']>;
  owner_not_contains?: InputMaybe<Scalars['String']>;
  owner_not_contains_nocase?: InputMaybe<Scalars['String']>;
  owner_not_ends_with?: InputMaybe<Scalars['String']>;
  owner_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  owner_not_in?: InputMaybe<Array<Scalars['String']>>;
  owner_not_starts_with?: InputMaybe<Scalars['String']>;
  owner_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  owner_starts_with?: InputMaybe<Scalars['String']>;
  owner_starts_with_nocase?: InputMaybe<Scalars['String']>;
  takeOrders_?: InputMaybe<TakeOrderEntity_Filter>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
  validInputs?: InputMaybe<Array<Scalars['String']>>;
  validInputs_?: InputMaybe<Io_Filter>;
  validInputs_contains?: InputMaybe<Array<Scalars['String']>>;
  validInputs_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  validInputs_not?: InputMaybe<Array<Scalars['String']>>;
  validInputs_not_contains?: InputMaybe<Array<Scalars['String']>>;
  validInputs_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  validOutputs?: InputMaybe<Array<Scalars['String']>>;
  validOutputs_?: InputMaybe<Io_Filter>;
  validOutputs_contains?: InputMaybe<Array<Scalars['String']>>;
  validOutputs_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  validOutputs_not?: InputMaybe<Array<Scalars['String']>>;
  validOutputs_not_contains?: InputMaybe<Array<Scalars['String']>>;
  validOutputs_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
};

export type Order_OrderBy =
  | 'emitter'
  | 'emitter__id'
  | 'expression'
  | 'expressionDeployer'
  | 'handleIO'
  | 'id'
  | 'interpreter'
  | 'interpreterStore'
  | 'meta'
  | 'meta__id'
  | 'meta__metaBytes'
  | 'orderActive'
  | 'orderHash'
  | 'orderJSONString'
  | 'ordersClears'
  | 'owner'
  | 'owner__id'
  | 'takeOrders'
  | 'timestamp'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp'
  | 'validInputs'
  | 'validOutputs';

export type Query = {
  __typename?: 'Query';
  /** Access to subgraph metadata */
  _meta?: Maybe<_Meta_>;
  account?: Maybe<Account>;
  accounts: Array<Account>;
  bounties: Array<Bounty>;
  bounty?: Maybe<Bounty>;
  clearOrderConfig?: Maybe<ClearOrderConfig>;
  clearOrderConfigs: Array<ClearOrderConfig>;
  contextEntities: Array<ContextEntity>;
  contextEntity?: Maybe<ContextEntity>;
  erc20?: Maybe<Erc20>;
  erc20S: Array<Erc20>;
  event?: Maybe<Event>;
  events: Array<Event>;
  io?: Maybe<Io>;
  ios: Array<Io>;
  metaContentV1?: Maybe<MetaContentV1>;
  metaContentV1S: Array<MetaContentV1>;
  order?: Maybe<Order>;
  orderBook?: Maybe<OrderBook>;
  orderBooks: Array<OrderBook>;
  orderClear?: Maybe<OrderClear>;
  orderClearStateChange?: Maybe<OrderClearStateChange>;
  orderClearStateChanges: Array<OrderClearStateChange>;
  orderClears: Array<OrderClear>;
  orders: Array<Order>;
  rainMetaV1?: Maybe<RainMetaV1>;
  rainMetaV1S: Array<RainMetaV1>;
  signedContext?: Maybe<SignedContext>;
  signedContexts: Array<SignedContext>;
  takeOrderEntities: Array<TakeOrderEntity>;
  takeOrderEntity?: Maybe<TakeOrderEntity>;
  tokenVault?: Maybe<TokenVault>;
  tokenVaults: Array<TokenVault>;
  transaction?: Maybe<Transaction>;
  transactions: Array<Transaction>;
  vault?: Maybe<Vault>;
  vaultDeposit?: Maybe<VaultDeposit>;
  vaultDeposits: Array<VaultDeposit>;
  vaultWithdraw?: Maybe<VaultWithdraw>;
  vaultWithdraws: Array<VaultWithdraw>;
  vaults: Array<Vault>;
};


export type Query_MetaArgs = {
  block?: InputMaybe<Block_Height>;
};


export type QueryAccountArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryAccountsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Account_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Account_Filter>;
};


export type QueryBountiesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Bounty_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Bounty_Filter>;
};


export type QueryBountyArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryClearOrderConfigArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryClearOrderConfigsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<ClearOrderConfig_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<ClearOrderConfig_Filter>;
};


export type QueryContextEntitiesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<ContextEntity_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<ContextEntity_Filter>;
};


export type QueryContextEntityArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryErc20Args = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryErc20SArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Erc20_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Erc20_Filter>;
};


export type QueryEventArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryEventsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Event_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Event_Filter>;
};


export type QueryIoArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryIosArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Io_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Io_Filter>;
};


export type QueryMetaContentV1Args = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryMetaContentV1SArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<MetaContentV1_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<MetaContentV1_Filter>;
};


export type QueryOrderArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryOrderBookArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryOrderBooksArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderBook_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<OrderBook_Filter>;
};


export type QueryOrderClearArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryOrderClearStateChangeArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryOrderClearStateChangesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClearStateChange_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<OrderClearStateChange_Filter>;
};


export type QueryOrderClearsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClear_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<OrderClear_Filter>;
};


export type QueryOrdersArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Order_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Order_Filter>;
};


export type QueryRainMetaV1Args = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryRainMetaV1SArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<RainMetaV1_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<RainMetaV1_Filter>;
};


export type QuerySignedContextArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QuerySignedContextsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<SignedContext_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<SignedContext_Filter>;
};


export type QueryTakeOrderEntitiesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TakeOrderEntity_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<TakeOrderEntity_Filter>;
};


export type QueryTakeOrderEntityArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryTokenVaultArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryTokenVaultsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TokenVault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<TokenVault_Filter>;
};


export type QueryTransactionArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryTransactionsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Transaction_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Transaction_Filter>;
};


export type QueryVaultArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryVaultDepositArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryVaultDepositsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultDeposit_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<VaultDeposit_Filter>;
};


export type QueryVaultWithdrawArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type QueryVaultWithdrawsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultWithdraw_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<VaultWithdraw_Filter>;
};


export type QueryVaultsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Vault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Vault_Filter>;
};

export type RainMetaV1 = {
  __typename?: 'RainMetaV1';
  /** The meta content V1 decoded from the meta bytes emitted */
  content?: Maybe<Array<MetaContentV1>>;
  /** Hash of the meta directly emitted by the contract */
  id: Scalars['Bytes'];
  /** Original meta bytes directly emitted from the contract */
  metaBytes: Scalars['Bytes'];
};


export type RainMetaV1ContentArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<MetaContentV1_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<MetaContentV1_Filter>;
};

export type RainMetaV1_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<RainMetaV1_Filter>>>;
  content_?: InputMaybe<MetaContentV1_Filter>;
  id?: InputMaybe<Scalars['Bytes']>;
  id_contains?: InputMaybe<Scalars['Bytes']>;
  id_gt?: InputMaybe<Scalars['Bytes']>;
  id_gte?: InputMaybe<Scalars['Bytes']>;
  id_in?: InputMaybe<Array<Scalars['Bytes']>>;
  id_lt?: InputMaybe<Scalars['Bytes']>;
  id_lte?: InputMaybe<Scalars['Bytes']>;
  id_not?: InputMaybe<Scalars['Bytes']>;
  id_not_contains?: InputMaybe<Scalars['Bytes']>;
  id_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  metaBytes?: InputMaybe<Scalars['Bytes']>;
  metaBytes_contains?: InputMaybe<Scalars['Bytes']>;
  metaBytes_gt?: InputMaybe<Scalars['Bytes']>;
  metaBytes_gte?: InputMaybe<Scalars['Bytes']>;
  metaBytes_in?: InputMaybe<Array<Scalars['Bytes']>>;
  metaBytes_lt?: InputMaybe<Scalars['Bytes']>;
  metaBytes_lte?: InputMaybe<Scalars['Bytes']>;
  metaBytes_not?: InputMaybe<Scalars['Bytes']>;
  metaBytes_not_contains?: InputMaybe<Scalars['Bytes']>;
  metaBytes_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
  or?: InputMaybe<Array<InputMaybe<RainMetaV1_Filter>>>;
};

export type RainMetaV1_OrderBy =
  | 'content'
  | 'id'
  | 'metaBytes';

export type SignedContext = {
  __typename?: 'SignedContext';
  context?: Maybe<Array<Scalars['BigInt']>>;
  id: Scalars['ID'];
  signer: Scalars['Bytes'];
};

export type SignedContext_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<SignedContext_Filter>>>;
  context?: InputMaybe<Array<Scalars['BigInt']>>;
  context_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  context_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  context_not?: InputMaybe<Array<Scalars['BigInt']>>;
  context_not_contains?: InputMaybe<Array<Scalars['BigInt']>>;
  context_not_contains_nocase?: InputMaybe<Array<Scalars['BigInt']>>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<SignedContext_Filter>>>;
  signer?: InputMaybe<Scalars['Bytes']>;
  signer_contains?: InputMaybe<Scalars['Bytes']>;
  signer_gt?: InputMaybe<Scalars['Bytes']>;
  signer_gte?: InputMaybe<Scalars['Bytes']>;
  signer_in?: InputMaybe<Array<Scalars['Bytes']>>;
  signer_lt?: InputMaybe<Scalars['Bytes']>;
  signer_lte?: InputMaybe<Scalars['Bytes']>;
  signer_not?: InputMaybe<Scalars['Bytes']>;
  signer_not_contains?: InputMaybe<Scalars['Bytes']>;
  signer_not_in?: InputMaybe<Array<Scalars['Bytes']>>;
};

export type SignedContext_OrderBy =
  | 'context'
  | 'id'
  | 'signer';

export type Subscription = {
  __typename?: 'Subscription';
  /** Access to subgraph metadata */
  _meta?: Maybe<_Meta_>;
  account?: Maybe<Account>;
  accounts: Array<Account>;
  bounties: Array<Bounty>;
  bounty?: Maybe<Bounty>;
  clearOrderConfig?: Maybe<ClearOrderConfig>;
  clearOrderConfigs: Array<ClearOrderConfig>;
  contextEntities: Array<ContextEntity>;
  contextEntity?: Maybe<ContextEntity>;
  erc20?: Maybe<Erc20>;
  erc20S: Array<Erc20>;
  event?: Maybe<Event>;
  events: Array<Event>;
  io?: Maybe<Io>;
  ios: Array<Io>;
  metaContentV1?: Maybe<MetaContentV1>;
  metaContentV1S: Array<MetaContentV1>;
  order?: Maybe<Order>;
  orderBook?: Maybe<OrderBook>;
  orderBooks: Array<OrderBook>;
  orderClear?: Maybe<OrderClear>;
  orderClearStateChange?: Maybe<OrderClearStateChange>;
  orderClearStateChanges: Array<OrderClearStateChange>;
  orderClears: Array<OrderClear>;
  orders: Array<Order>;
  rainMetaV1?: Maybe<RainMetaV1>;
  rainMetaV1S: Array<RainMetaV1>;
  signedContext?: Maybe<SignedContext>;
  signedContexts: Array<SignedContext>;
  takeOrderEntities: Array<TakeOrderEntity>;
  takeOrderEntity?: Maybe<TakeOrderEntity>;
  tokenVault?: Maybe<TokenVault>;
  tokenVaults: Array<TokenVault>;
  transaction?: Maybe<Transaction>;
  transactions: Array<Transaction>;
  vault?: Maybe<Vault>;
  vaultDeposit?: Maybe<VaultDeposit>;
  vaultDeposits: Array<VaultDeposit>;
  vaultWithdraw?: Maybe<VaultWithdraw>;
  vaultWithdraws: Array<VaultWithdraw>;
  vaults: Array<Vault>;
};


export type Subscription_MetaArgs = {
  block?: InputMaybe<Block_Height>;
};


export type SubscriptionAccountArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionAccountsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Account_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Account_Filter>;
};


export type SubscriptionBountiesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Bounty_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Bounty_Filter>;
};


export type SubscriptionBountyArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionClearOrderConfigArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionClearOrderConfigsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<ClearOrderConfig_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<ClearOrderConfig_Filter>;
};


export type SubscriptionContextEntitiesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<ContextEntity_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<ContextEntity_Filter>;
};


export type SubscriptionContextEntityArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionErc20Args = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionErc20SArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Erc20_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Erc20_Filter>;
};


export type SubscriptionEventArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionEventsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Event_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Event_Filter>;
};


export type SubscriptionIoArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionIosArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Io_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Io_Filter>;
};


export type SubscriptionMetaContentV1Args = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionMetaContentV1SArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<MetaContentV1_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<MetaContentV1_Filter>;
};


export type SubscriptionOrderArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionOrderBookArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionOrderBooksArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderBook_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<OrderBook_Filter>;
};


export type SubscriptionOrderClearArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionOrderClearStateChangeArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionOrderClearStateChangesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClearStateChange_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<OrderClearStateChange_Filter>;
};


export type SubscriptionOrderClearsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClear_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<OrderClear_Filter>;
};


export type SubscriptionOrdersArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Order_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Order_Filter>;
};


export type SubscriptionRainMetaV1Args = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionRainMetaV1SArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<RainMetaV1_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<RainMetaV1_Filter>;
};


export type SubscriptionSignedContextArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionSignedContextsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<SignedContext_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<SignedContext_Filter>;
};


export type SubscriptionTakeOrderEntitiesArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TakeOrderEntity_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<TakeOrderEntity_Filter>;
};


export type SubscriptionTakeOrderEntityArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionTokenVaultArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionTokenVaultsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TokenVault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<TokenVault_Filter>;
};


export type SubscriptionTransactionArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionTransactionsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Transaction_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Transaction_Filter>;
};


export type SubscriptionVaultArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionVaultDepositArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionVaultDepositsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultDeposit_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<VaultDeposit_Filter>;
};


export type SubscriptionVaultWithdrawArgs = {
  block?: InputMaybe<Block_Height>;
  id: Scalars['ID'];
  subgraphError?: _SubgraphErrorPolicy_;
};


export type SubscriptionVaultWithdrawsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultWithdraw_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<VaultWithdraw_Filter>;
};


export type SubscriptionVaultsArgs = {
  block?: InputMaybe<Block_Height>;
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Vault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  subgraphError?: _SubgraphErrorPolicy_;
  where?: InputMaybe<Vault_Filter>;
};

export type TakeOrderEntity = Event & {
  __typename?: 'TakeOrderEntity';
  /** IO Ratio */
  IORatio: Scalars['BigDecimal'];
  context?: Maybe<ContextEntity>;
  emitter: Account;
  id: Scalars['ID'];
  /** The input amount from the perspective of sender */
  input: Scalars['BigInt'];
  inputDisplay: Scalars['BigDecimal'];
  /** The index of the input token in order to match with the take order output */
  inputIOIndex: Scalars['BigInt'];
  /** Input token from the perspective of the order taker */
  inputToken: Erc20;
  order: Order;
  /** The output amount from the perspective of sender */
  output: Scalars['BigInt'];
  outputDisplay: Scalars['BigDecimal'];
  /** The index of the output token in order to match with the take order input. */
  outputIOIndex: Scalars['BigInt'];
  /** Output token from the perspective of the order taker */
  outputToken: Erc20;
  sender: Account;
  timestamp: Scalars['BigInt'];
  transaction: Transaction;
};

export type TakeOrderEntity_Filter = {
  IORatio?: InputMaybe<Scalars['BigDecimal']>;
  IORatio_gt?: InputMaybe<Scalars['BigDecimal']>;
  IORatio_gte?: InputMaybe<Scalars['BigDecimal']>;
  IORatio_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  IORatio_lt?: InputMaybe<Scalars['BigDecimal']>;
  IORatio_lte?: InputMaybe<Scalars['BigDecimal']>;
  IORatio_not?: InputMaybe<Scalars['BigDecimal']>;
  IORatio_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<TakeOrderEntity_Filter>>>;
  context?: InputMaybe<Scalars['String']>;
  context_?: InputMaybe<ContextEntity_Filter>;
  context_contains?: InputMaybe<Scalars['String']>;
  context_contains_nocase?: InputMaybe<Scalars['String']>;
  context_ends_with?: InputMaybe<Scalars['String']>;
  context_ends_with_nocase?: InputMaybe<Scalars['String']>;
  context_gt?: InputMaybe<Scalars['String']>;
  context_gte?: InputMaybe<Scalars['String']>;
  context_in?: InputMaybe<Array<Scalars['String']>>;
  context_lt?: InputMaybe<Scalars['String']>;
  context_lte?: InputMaybe<Scalars['String']>;
  context_not?: InputMaybe<Scalars['String']>;
  context_not_contains?: InputMaybe<Scalars['String']>;
  context_not_contains_nocase?: InputMaybe<Scalars['String']>;
  context_not_ends_with?: InputMaybe<Scalars['String']>;
  context_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  context_not_in?: InputMaybe<Array<Scalars['String']>>;
  context_not_starts_with?: InputMaybe<Scalars['String']>;
  context_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  context_starts_with?: InputMaybe<Scalars['String']>;
  context_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  input?: InputMaybe<Scalars['BigInt']>;
  inputDisplay?: InputMaybe<Scalars['BigDecimal']>;
  inputDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  inputDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  inputDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  inputDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  inputDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  inputDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  inputDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  inputIOIndex?: InputMaybe<Scalars['BigInt']>;
  inputIOIndex_gt?: InputMaybe<Scalars['BigInt']>;
  inputIOIndex_gte?: InputMaybe<Scalars['BigInt']>;
  inputIOIndex_in?: InputMaybe<Array<Scalars['BigInt']>>;
  inputIOIndex_lt?: InputMaybe<Scalars['BigInt']>;
  inputIOIndex_lte?: InputMaybe<Scalars['BigInt']>;
  inputIOIndex_not?: InputMaybe<Scalars['BigInt']>;
  inputIOIndex_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  inputToken?: InputMaybe<Scalars['String']>;
  inputToken_?: InputMaybe<Erc20_Filter>;
  inputToken_contains?: InputMaybe<Scalars['String']>;
  inputToken_contains_nocase?: InputMaybe<Scalars['String']>;
  inputToken_ends_with?: InputMaybe<Scalars['String']>;
  inputToken_ends_with_nocase?: InputMaybe<Scalars['String']>;
  inputToken_gt?: InputMaybe<Scalars['String']>;
  inputToken_gte?: InputMaybe<Scalars['String']>;
  inputToken_in?: InputMaybe<Array<Scalars['String']>>;
  inputToken_lt?: InputMaybe<Scalars['String']>;
  inputToken_lte?: InputMaybe<Scalars['String']>;
  inputToken_not?: InputMaybe<Scalars['String']>;
  inputToken_not_contains?: InputMaybe<Scalars['String']>;
  inputToken_not_contains_nocase?: InputMaybe<Scalars['String']>;
  inputToken_not_ends_with?: InputMaybe<Scalars['String']>;
  inputToken_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  inputToken_not_in?: InputMaybe<Array<Scalars['String']>>;
  inputToken_not_starts_with?: InputMaybe<Scalars['String']>;
  inputToken_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  inputToken_starts_with?: InputMaybe<Scalars['String']>;
  inputToken_starts_with_nocase?: InputMaybe<Scalars['String']>;
  input_gt?: InputMaybe<Scalars['BigInt']>;
  input_gte?: InputMaybe<Scalars['BigInt']>;
  input_in?: InputMaybe<Array<Scalars['BigInt']>>;
  input_lt?: InputMaybe<Scalars['BigInt']>;
  input_lte?: InputMaybe<Scalars['BigInt']>;
  input_not?: InputMaybe<Scalars['BigInt']>;
  input_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  or?: InputMaybe<Array<InputMaybe<TakeOrderEntity_Filter>>>;
  order?: InputMaybe<Scalars['String']>;
  order_?: InputMaybe<Order_Filter>;
  order_contains?: InputMaybe<Scalars['String']>;
  order_contains_nocase?: InputMaybe<Scalars['String']>;
  order_ends_with?: InputMaybe<Scalars['String']>;
  order_ends_with_nocase?: InputMaybe<Scalars['String']>;
  order_gt?: InputMaybe<Scalars['String']>;
  order_gte?: InputMaybe<Scalars['String']>;
  order_in?: InputMaybe<Array<Scalars['String']>>;
  order_lt?: InputMaybe<Scalars['String']>;
  order_lte?: InputMaybe<Scalars['String']>;
  order_not?: InputMaybe<Scalars['String']>;
  order_not_contains?: InputMaybe<Scalars['String']>;
  order_not_contains_nocase?: InputMaybe<Scalars['String']>;
  order_not_ends_with?: InputMaybe<Scalars['String']>;
  order_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  order_not_in?: InputMaybe<Array<Scalars['String']>>;
  order_not_starts_with?: InputMaybe<Scalars['String']>;
  order_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  order_starts_with?: InputMaybe<Scalars['String']>;
  order_starts_with_nocase?: InputMaybe<Scalars['String']>;
  output?: InputMaybe<Scalars['BigInt']>;
  outputDisplay?: InputMaybe<Scalars['BigDecimal']>;
  outputDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  outputDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  outputDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  outputDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  outputDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  outputDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  outputDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  outputIOIndex?: InputMaybe<Scalars['BigInt']>;
  outputIOIndex_gt?: InputMaybe<Scalars['BigInt']>;
  outputIOIndex_gte?: InputMaybe<Scalars['BigInt']>;
  outputIOIndex_in?: InputMaybe<Array<Scalars['BigInt']>>;
  outputIOIndex_lt?: InputMaybe<Scalars['BigInt']>;
  outputIOIndex_lte?: InputMaybe<Scalars['BigInt']>;
  outputIOIndex_not?: InputMaybe<Scalars['BigInt']>;
  outputIOIndex_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  outputToken?: InputMaybe<Scalars['String']>;
  outputToken_?: InputMaybe<Erc20_Filter>;
  outputToken_contains?: InputMaybe<Scalars['String']>;
  outputToken_contains_nocase?: InputMaybe<Scalars['String']>;
  outputToken_ends_with?: InputMaybe<Scalars['String']>;
  outputToken_ends_with_nocase?: InputMaybe<Scalars['String']>;
  outputToken_gt?: InputMaybe<Scalars['String']>;
  outputToken_gte?: InputMaybe<Scalars['String']>;
  outputToken_in?: InputMaybe<Array<Scalars['String']>>;
  outputToken_lt?: InputMaybe<Scalars['String']>;
  outputToken_lte?: InputMaybe<Scalars['String']>;
  outputToken_not?: InputMaybe<Scalars['String']>;
  outputToken_not_contains?: InputMaybe<Scalars['String']>;
  outputToken_not_contains_nocase?: InputMaybe<Scalars['String']>;
  outputToken_not_ends_with?: InputMaybe<Scalars['String']>;
  outputToken_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  outputToken_not_in?: InputMaybe<Array<Scalars['String']>>;
  outputToken_not_starts_with?: InputMaybe<Scalars['String']>;
  outputToken_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  outputToken_starts_with?: InputMaybe<Scalars['String']>;
  outputToken_starts_with_nocase?: InputMaybe<Scalars['String']>;
  output_gt?: InputMaybe<Scalars['BigInt']>;
  output_gte?: InputMaybe<Scalars['BigInt']>;
  output_in?: InputMaybe<Array<Scalars['BigInt']>>;
  output_lt?: InputMaybe<Scalars['BigInt']>;
  output_lte?: InputMaybe<Scalars['BigInt']>;
  output_not?: InputMaybe<Scalars['BigInt']>;
  output_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  sender?: InputMaybe<Scalars['String']>;
  sender_?: InputMaybe<Account_Filter>;
  sender_contains?: InputMaybe<Scalars['String']>;
  sender_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_ends_with?: InputMaybe<Scalars['String']>;
  sender_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_gt?: InputMaybe<Scalars['String']>;
  sender_gte?: InputMaybe<Scalars['String']>;
  sender_in?: InputMaybe<Array<Scalars['String']>>;
  sender_lt?: InputMaybe<Scalars['String']>;
  sender_lte?: InputMaybe<Scalars['String']>;
  sender_not?: InputMaybe<Scalars['String']>;
  sender_not_contains?: InputMaybe<Scalars['String']>;
  sender_not_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_not_ends_with?: InputMaybe<Scalars['String']>;
  sender_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_not_in?: InputMaybe<Array<Scalars['String']>>;
  sender_not_starts_with?: InputMaybe<Scalars['String']>;
  sender_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  sender_starts_with?: InputMaybe<Scalars['String']>;
  sender_starts_with_nocase?: InputMaybe<Scalars['String']>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type TakeOrderEntity_OrderBy =
  | 'IORatio'
  | 'context'
  | 'context__id'
  | 'context__timestamp'
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'input'
  | 'inputDisplay'
  | 'inputIOIndex'
  | 'inputToken'
  | 'inputToken__decimals'
  | 'inputToken__id'
  | 'inputToken__name'
  | 'inputToken__symbol'
  | 'inputToken__totalSupply'
  | 'inputToken__totalSupplyDisplay'
  | 'order'
  | 'order__expression'
  | 'order__expressionDeployer'
  | 'order__handleIO'
  | 'order__id'
  | 'order__interpreter'
  | 'order__interpreterStore'
  | 'order__orderActive'
  | 'order__orderHash'
  | 'order__orderJSONString'
  | 'order__timestamp'
  | 'output'
  | 'outputDisplay'
  | 'outputIOIndex'
  | 'outputToken'
  | 'outputToken__decimals'
  | 'outputToken__id'
  | 'outputToken__name'
  | 'outputToken__symbol'
  | 'outputToken__totalSupply'
  | 'outputToken__totalSupplyDisplay'
  | 'sender'
  | 'sender__id'
  | 'timestamp'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp';

export type TokenVault = {
  __typename?: 'TokenVault';
  /** The balance of this token, for this vault, for this owner */
  balance: Scalars['BigInt'];
  balanceDisplay: Scalars['BigDecimal'];
  id: Scalars['ID'];
  orderClears?: Maybe<Array<OrderClear>>;
  /** Orders that reference this vault, owner and token */
  orders?: Maybe<Array<Order>>;
  /** The owner of this Vault */
  owner: Account;
  /** The token that has a balance for this vault and owner. */
  token: Erc20;
  /** The id of this vault */
  vault: Vault;
  vaultId: Scalars['BigInt'];
};


export type TokenVaultOrderClearsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<OrderClear_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<OrderClear_Filter>;
};


export type TokenVaultOrdersArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Order_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Order_Filter>;
};

export type TokenVault_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<TokenVault_Filter>>>;
  balance?: InputMaybe<Scalars['BigInt']>;
  balanceDisplay?: InputMaybe<Scalars['BigDecimal']>;
  balanceDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  balanceDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  balanceDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  balanceDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  balanceDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  balanceDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  balanceDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  balance_gt?: InputMaybe<Scalars['BigInt']>;
  balance_gte?: InputMaybe<Scalars['BigInt']>;
  balance_in?: InputMaybe<Array<Scalars['BigInt']>>;
  balance_lt?: InputMaybe<Scalars['BigInt']>;
  balance_lte?: InputMaybe<Scalars['BigInt']>;
  balance_not?: InputMaybe<Scalars['BigInt']>;
  balance_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<TokenVault_Filter>>>;
  orderClears?: InputMaybe<Array<Scalars['String']>>;
  orderClears_?: InputMaybe<OrderClear_Filter>;
  orderClears_contains?: InputMaybe<Array<Scalars['String']>>;
  orderClears_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  orderClears_not?: InputMaybe<Array<Scalars['String']>>;
  orderClears_not_contains?: InputMaybe<Array<Scalars['String']>>;
  orderClears_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  orders?: InputMaybe<Array<Scalars['String']>>;
  orders_?: InputMaybe<Order_Filter>;
  orders_contains?: InputMaybe<Array<Scalars['String']>>;
  orders_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  orders_not?: InputMaybe<Array<Scalars['String']>>;
  orders_not_contains?: InputMaybe<Array<Scalars['String']>>;
  orders_not_contains_nocase?: InputMaybe<Array<Scalars['String']>>;
  owner?: InputMaybe<Scalars['String']>;
  owner_?: InputMaybe<Account_Filter>;
  owner_contains?: InputMaybe<Scalars['String']>;
  owner_contains_nocase?: InputMaybe<Scalars['String']>;
  owner_ends_with?: InputMaybe<Scalars['String']>;
  owner_ends_with_nocase?: InputMaybe<Scalars['String']>;
  owner_gt?: InputMaybe<Scalars['String']>;
  owner_gte?: InputMaybe<Scalars['String']>;
  owner_in?: InputMaybe<Array<Scalars['String']>>;
  owner_lt?: InputMaybe<Scalars['String']>;
  owner_lte?: InputMaybe<Scalars['String']>;
  owner_not?: InputMaybe<Scalars['String']>;
  owner_not_contains?: InputMaybe<Scalars['String']>;
  owner_not_contains_nocase?: InputMaybe<Scalars['String']>;
  owner_not_ends_with?: InputMaybe<Scalars['String']>;
  owner_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  owner_not_in?: InputMaybe<Array<Scalars['String']>>;
  owner_not_starts_with?: InputMaybe<Scalars['String']>;
  owner_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  owner_starts_with?: InputMaybe<Scalars['String']>;
  owner_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token?: InputMaybe<Scalars['String']>;
  token_?: InputMaybe<Erc20_Filter>;
  token_contains?: InputMaybe<Scalars['String']>;
  token_contains_nocase?: InputMaybe<Scalars['String']>;
  token_ends_with?: InputMaybe<Scalars['String']>;
  token_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_gt?: InputMaybe<Scalars['String']>;
  token_gte?: InputMaybe<Scalars['String']>;
  token_in?: InputMaybe<Array<Scalars['String']>>;
  token_lt?: InputMaybe<Scalars['String']>;
  token_lte?: InputMaybe<Scalars['String']>;
  token_not?: InputMaybe<Scalars['String']>;
  token_not_contains?: InputMaybe<Scalars['String']>;
  token_not_contains_nocase?: InputMaybe<Scalars['String']>;
  token_not_ends_with?: InputMaybe<Scalars['String']>;
  token_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_not_in?: InputMaybe<Array<Scalars['String']>>;
  token_not_starts_with?: InputMaybe<Scalars['String']>;
  token_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_starts_with?: InputMaybe<Scalars['String']>;
  token_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault?: InputMaybe<Scalars['String']>;
  vaultId?: InputMaybe<Scalars['BigInt']>;
  vaultId_gt?: InputMaybe<Scalars['BigInt']>;
  vaultId_gte?: InputMaybe<Scalars['BigInt']>;
  vaultId_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultId_lt?: InputMaybe<Scalars['BigInt']>;
  vaultId_lte?: InputMaybe<Scalars['BigInt']>;
  vaultId_not?: InputMaybe<Scalars['BigInt']>;
  vaultId_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vault_?: InputMaybe<Vault_Filter>;
  vault_contains?: InputMaybe<Scalars['String']>;
  vault_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_ends_with?: InputMaybe<Scalars['String']>;
  vault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_gt?: InputMaybe<Scalars['String']>;
  vault_gte?: InputMaybe<Scalars['String']>;
  vault_in?: InputMaybe<Array<Scalars['String']>>;
  vault_lt?: InputMaybe<Scalars['String']>;
  vault_lte?: InputMaybe<Scalars['String']>;
  vault_not?: InputMaybe<Scalars['String']>;
  vault_not_contains?: InputMaybe<Scalars['String']>;
  vault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_not_ends_with?: InputMaybe<Scalars['String']>;
  vault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_not_in?: InputMaybe<Array<Scalars['String']>>;
  vault_not_starts_with?: InputMaybe<Scalars['String']>;
  vault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault_starts_with?: InputMaybe<Scalars['String']>;
  vault_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type TokenVault_OrderBy =
  | 'balance'
  | 'balanceDisplay'
  | 'id'
  | 'orderClears'
  | 'orders'
  | 'owner'
  | 'owner__id'
  | 'token'
  | 'token__decimals'
  | 'token__id'
  | 'token__name'
  | 'token__symbol'
  | 'token__totalSupply'
  | 'token__totalSupplyDisplay'
  | 'vault'
  | 'vaultId'
  | 'vault__id'
  | 'vault__vaultId';

export type Transaction = {
  __typename?: 'Transaction';
  blockNumber: Scalars['BigInt'];
  events?: Maybe<Array<Event>>;
  id: Scalars['ID'];
  timestamp: Scalars['BigInt'];
};


export type TransactionEventsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<Event_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<Event_Filter>;
};

export type Transaction_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Transaction_Filter>>>;
  blockNumber?: InputMaybe<Scalars['BigInt']>;
  blockNumber_gt?: InputMaybe<Scalars['BigInt']>;
  blockNumber_gte?: InputMaybe<Scalars['BigInt']>;
  blockNumber_in?: InputMaybe<Array<Scalars['BigInt']>>;
  blockNumber_lt?: InputMaybe<Scalars['BigInt']>;
  blockNumber_lte?: InputMaybe<Scalars['BigInt']>;
  blockNumber_not?: InputMaybe<Scalars['BigInt']>;
  blockNumber_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  events_?: InputMaybe<Event_Filter>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<Transaction_Filter>>>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
};

export type Transaction_OrderBy =
  | 'blockNumber'
  | 'events'
  | 'id'
  | 'timestamp';

export type Vault = {
  __typename?: 'Vault';
  /** Deposits into this Vault */
  deposits?: Maybe<Array<VaultDeposit>>;
  id: Scalars['ID'];
  /** The owner of this Vault */
  owner: Account;
  /** Tokens in this Vault */
  tokenVaults?: Maybe<Array<TokenVault>>;
  vaultId: Scalars['BigInt'];
  /** Withdrawals from this Vault */
  withdraws?: Maybe<Array<VaultWithdraw>>;
};


export type VaultDepositsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultDeposit_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<VaultDeposit_Filter>;
};


export type VaultTokenVaultsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<TokenVault_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<TokenVault_Filter>;
};


export type VaultWithdrawsArgs = {
  first?: InputMaybe<Scalars['Int']>;
  orderBy?: InputMaybe<VaultWithdraw_OrderBy>;
  orderDirection?: InputMaybe<OrderDirection>;
  skip?: InputMaybe<Scalars['Int']>;
  where?: InputMaybe<VaultWithdraw_Filter>;
};

export type VaultDeposit = Event & {
  __typename?: 'VaultDeposit';
  /** The amount that was deposited */
  amount: Scalars['BigInt'];
  amountDisplay: Scalars['BigDecimal'];
  emitter: Account;
  id: Scalars['ID'];
  /** The transaction sender of this deposit */
  sender: Account;
  timestamp: Scalars['BigInt'];
  /** The token that was deposited */
  token: Erc20;
  /** The current balance of this token for this Vault */
  tokenVault: TokenVault;
  transaction: Transaction;
  /** The Vault that was deposited into */
  vault: Vault;
  /** The vaultId that was deposited into */
  vaultId: Scalars['BigInt'];
};

export type VaultDeposit_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  amount?: InputMaybe<Scalars['BigInt']>;
  amountDisplay?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  amountDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  amount_gt?: InputMaybe<Scalars['BigInt']>;
  amount_gte?: InputMaybe<Scalars['BigInt']>;
  amount_in?: InputMaybe<Array<Scalars['BigInt']>>;
  amount_lt?: InputMaybe<Scalars['BigInt']>;
  amount_lte?: InputMaybe<Scalars['BigInt']>;
  amount_not?: InputMaybe<Scalars['BigInt']>;
  amount_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  and?: InputMaybe<Array<InputMaybe<VaultDeposit_Filter>>>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<VaultDeposit_Filter>>>;
  sender?: InputMaybe<Scalars['String']>;
  sender_?: InputMaybe<Account_Filter>;
  sender_contains?: InputMaybe<Scalars['String']>;
  sender_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_ends_with?: InputMaybe<Scalars['String']>;
  sender_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_gt?: InputMaybe<Scalars['String']>;
  sender_gte?: InputMaybe<Scalars['String']>;
  sender_in?: InputMaybe<Array<Scalars['String']>>;
  sender_lt?: InputMaybe<Scalars['String']>;
  sender_lte?: InputMaybe<Scalars['String']>;
  sender_not?: InputMaybe<Scalars['String']>;
  sender_not_contains?: InputMaybe<Scalars['String']>;
  sender_not_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_not_ends_with?: InputMaybe<Scalars['String']>;
  sender_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_not_in?: InputMaybe<Array<Scalars['String']>>;
  sender_not_starts_with?: InputMaybe<Scalars['String']>;
  sender_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  sender_starts_with?: InputMaybe<Scalars['String']>;
  sender_starts_with_nocase?: InputMaybe<Scalars['String']>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  token?: InputMaybe<Scalars['String']>;
  tokenVault?: InputMaybe<Scalars['String']>;
  tokenVault_?: InputMaybe<TokenVault_Filter>;
  tokenVault_contains?: InputMaybe<Scalars['String']>;
  tokenVault_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_ends_with?: InputMaybe<Scalars['String']>;
  tokenVault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_gt?: InputMaybe<Scalars['String']>;
  tokenVault_gte?: InputMaybe<Scalars['String']>;
  tokenVault_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVault_lt?: InputMaybe<Scalars['String']>;
  tokenVault_lte?: InputMaybe<Scalars['String']>;
  tokenVault_not?: InputMaybe<Scalars['String']>;
  tokenVault_not_contains?: InputMaybe<Scalars['String']>;
  tokenVault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_not_ends_with?: InputMaybe<Scalars['String']>;
  tokenVault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_not_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVault_not_starts_with?: InputMaybe<Scalars['String']>;
  tokenVault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_starts_with?: InputMaybe<Scalars['String']>;
  tokenVault_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_?: InputMaybe<Erc20_Filter>;
  token_contains?: InputMaybe<Scalars['String']>;
  token_contains_nocase?: InputMaybe<Scalars['String']>;
  token_ends_with?: InputMaybe<Scalars['String']>;
  token_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_gt?: InputMaybe<Scalars['String']>;
  token_gte?: InputMaybe<Scalars['String']>;
  token_in?: InputMaybe<Array<Scalars['String']>>;
  token_lt?: InputMaybe<Scalars['String']>;
  token_lte?: InputMaybe<Scalars['String']>;
  token_not?: InputMaybe<Scalars['String']>;
  token_not_contains?: InputMaybe<Scalars['String']>;
  token_not_contains_nocase?: InputMaybe<Scalars['String']>;
  token_not_ends_with?: InputMaybe<Scalars['String']>;
  token_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_not_in?: InputMaybe<Array<Scalars['String']>>;
  token_not_starts_with?: InputMaybe<Scalars['String']>;
  token_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_starts_with?: InputMaybe<Scalars['String']>;
  token_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault?: InputMaybe<Scalars['String']>;
  vaultId?: InputMaybe<Scalars['BigInt']>;
  vaultId_gt?: InputMaybe<Scalars['BigInt']>;
  vaultId_gte?: InputMaybe<Scalars['BigInt']>;
  vaultId_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultId_lt?: InputMaybe<Scalars['BigInt']>;
  vaultId_lte?: InputMaybe<Scalars['BigInt']>;
  vaultId_not?: InputMaybe<Scalars['BigInt']>;
  vaultId_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vault_?: InputMaybe<Vault_Filter>;
  vault_contains?: InputMaybe<Scalars['String']>;
  vault_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_ends_with?: InputMaybe<Scalars['String']>;
  vault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_gt?: InputMaybe<Scalars['String']>;
  vault_gte?: InputMaybe<Scalars['String']>;
  vault_in?: InputMaybe<Array<Scalars['String']>>;
  vault_lt?: InputMaybe<Scalars['String']>;
  vault_lte?: InputMaybe<Scalars['String']>;
  vault_not?: InputMaybe<Scalars['String']>;
  vault_not_contains?: InputMaybe<Scalars['String']>;
  vault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_not_ends_with?: InputMaybe<Scalars['String']>;
  vault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_not_in?: InputMaybe<Array<Scalars['String']>>;
  vault_not_starts_with?: InputMaybe<Scalars['String']>;
  vault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault_starts_with?: InputMaybe<Scalars['String']>;
  vault_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type VaultDeposit_OrderBy =
  | 'amount'
  | 'amountDisplay'
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'sender'
  | 'sender__id'
  | 'timestamp'
  | 'token'
  | 'tokenVault'
  | 'tokenVault__balance'
  | 'tokenVault__balanceDisplay'
  | 'tokenVault__id'
  | 'tokenVault__vaultId'
  | 'token__decimals'
  | 'token__id'
  | 'token__name'
  | 'token__symbol'
  | 'token__totalSupply'
  | 'token__totalSupplyDisplay'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp'
  | 'vault'
  | 'vaultId'
  | 'vault__id'
  | 'vault__vaultId';

export type VaultWithdraw = Event & {
  __typename?: 'VaultWithdraw';
  /** The amount that was withdrawn */
  amount: Scalars['BigInt'];
  amountDisplay: Scalars['BigDecimal'];
  emitter: Account;
  id: Scalars['ID'];
  /** The amount that was requested be withdrawn */
  requestedAmount: Scalars['BigInt'];
  requestedAmountDisplay: Scalars['BigDecimal'];
  /** The transaction sender of this withdrawal */
  sender: Account;
  timestamp: Scalars['BigInt'];
  /** The token that was withdrawn */
  token: Erc20;
  /** The current balance of this token for this Vault */
  tokenVault: TokenVault;
  transaction: Transaction;
  /** The Vault that was withdrawn from */
  vault: Vault;
  /** The vaultId that was withdrawn from */
  vaultId: Scalars['BigInt'];
};

export type VaultWithdraw_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  amount?: InputMaybe<Scalars['BigInt']>;
  amountDisplay?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  amountDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  amountDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  amount_gt?: InputMaybe<Scalars['BigInt']>;
  amount_gte?: InputMaybe<Scalars['BigInt']>;
  amount_in?: InputMaybe<Array<Scalars['BigInt']>>;
  amount_lt?: InputMaybe<Scalars['BigInt']>;
  amount_lte?: InputMaybe<Scalars['BigInt']>;
  amount_not?: InputMaybe<Scalars['BigInt']>;
  amount_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  and?: InputMaybe<Array<InputMaybe<VaultWithdraw_Filter>>>;
  emitter?: InputMaybe<Scalars['String']>;
  emitter_?: InputMaybe<Account_Filter>;
  emitter_contains?: InputMaybe<Scalars['String']>;
  emitter_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_ends_with?: InputMaybe<Scalars['String']>;
  emitter_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_gt?: InputMaybe<Scalars['String']>;
  emitter_gte?: InputMaybe<Scalars['String']>;
  emitter_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_lt?: InputMaybe<Scalars['String']>;
  emitter_lte?: InputMaybe<Scalars['String']>;
  emitter_not?: InputMaybe<Scalars['String']>;
  emitter_not_contains?: InputMaybe<Scalars['String']>;
  emitter_not_contains_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with?: InputMaybe<Scalars['String']>;
  emitter_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_not_in?: InputMaybe<Array<Scalars['String']>>;
  emitter_not_starts_with?: InputMaybe<Scalars['String']>;
  emitter_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  emitter_starts_with?: InputMaybe<Scalars['String']>;
  emitter_starts_with_nocase?: InputMaybe<Scalars['String']>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<VaultWithdraw_Filter>>>;
  requestedAmount?: InputMaybe<Scalars['BigInt']>;
  requestedAmountDisplay?: InputMaybe<Scalars['BigDecimal']>;
  requestedAmountDisplay_gt?: InputMaybe<Scalars['BigDecimal']>;
  requestedAmountDisplay_gte?: InputMaybe<Scalars['BigDecimal']>;
  requestedAmountDisplay_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  requestedAmountDisplay_lt?: InputMaybe<Scalars['BigDecimal']>;
  requestedAmountDisplay_lte?: InputMaybe<Scalars['BigDecimal']>;
  requestedAmountDisplay_not?: InputMaybe<Scalars['BigDecimal']>;
  requestedAmountDisplay_not_in?: InputMaybe<Array<Scalars['BigDecimal']>>;
  requestedAmount_gt?: InputMaybe<Scalars['BigInt']>;
  requestedAmount_gte?: InputMaybe<Scalars['BigInt']>;
  requestedAmount_in?: InputMaybe<Array<Scalars['BigInt']>>;
  requestedAmount_lt?: InputMaybe<Scalars['BigInt']>;
  requestedAmount_lte?: InputMaybe<Scalars['BigInt']>;
  requestedAmount_not?: InputMaybe<Scalars['BigInt']>;
  requestedAmount_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  sender?: InputMaybe<Scalars['String']>;
  sender_?: InputMaybe<Account_Filter>;
  sender_contains?: InputMaybe<Scalars['String']>;
  sender_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_ends_with?: InputMaybe<Scalars['String']>;
  sender_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_gt?: InputMaybe<Scalars['String']>;
  sender_gte?: InputMaybe<Scalars['String']>;
  sender_in?: InputMaybe<Array<Scalars['String']>>;
  sender_lt?: InputMaybe<Scalars['String']>;
  sender_lte?: InputMaybe<Scalars['String']>;
  sender_not?: InputMaybe<Scalars['String']>;
  sender_not_contains?: InputMaybe<Scalars['String']>;
  sender_not_contains_nocase?: InputMaybe<Scalars['String']>;
  sender_not_ends_with?: InputMaybe<Scalars['String']>;
  sender_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  sender_not_in?: InputMaybe<Array<Scalars['String']>>;
  sender_not_starts_with?: InputMaybe<Scalars['String']>;
  sender_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  sender_starts_with?: InputMaybe<Scalars['String']>;
  sender_starts_with_nocase?: InputMaybe<Scalars['String']>;
  timestamp?: InputMaybe<Scalars['BigInt']>;
  timestamp_gt?: InputMaybe<Scalars['BigInt']>;
  timestamp_gte?: InputMaybe<Scalars['BigInt']>;
  timestamp_in?: InputMaybe<Array<Scalars['BigInt']>>;
  timestamp_lt?: InputMaybe<Scalars['BigInt']>;
  timestamp_lte?: InputMaybe<Scalars['BigInt']>;
  timestamp_not?: InputMaybe<Scalars['BigInt']>;
  timestamp_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  token?: InputMaybe<Scalars['String']>;
  tokenVault?: InputMaybe<Scalars['String']>;
  tokenVault_?: InputMaybe<TokenVault_Filter>;
  tokenVault_contains?: InputMaybe<Scalars['String']>;
  tokenVault_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_ends_with?: InputMaybe<Scalars['String']>;
  tokenVault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_gt?: InputMaybe<Scalars['String']>;
  tokenVault_gte?: InputMaybe<Scalars['String']>;
  tokenVault_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVault_lt?: InputMaybe<Scalars['String']>;
  tokenVault_lte?: InputMaybe<Scalars['String']>;
  tokenVault_not?: InputMaybe<Scalars['String']>;
  tokenVault_not_contains?: InputMaybe<Scalars['String']>;
  tokenVault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_not_ends_with?: InputMaybe<Scalars['String']>;
  tokenVault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_not_in?: InputMaybe<Array<Scalars['String']>>;
  tokenVault_not_starts_with?: InputMaybe<Scalars['String']>;
  tokenVault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVault_starts_with?: InputMaybe<Scalars['String']>;
  tokenVault_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_?: InputMaybe<Erc20_Filter>;
  token_contains?: InputMaybe<Scalars['String']>;
  token_contains_nocase?: InputMaybe<Scalars['String']>;
  token_ends_with?: InputMaybe<Scalars['String']>;
  token_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_gt?: InputMaybe<Scalars['String']>;
  token_gte?: InputMaybe<Scalars['String']>;
  token_in?: InputMaybe<Array<Scalars['String']>>;
  token_lt?: InputMaybe<Scalars['String']>;
  token_lte?: InputMaybe<Scalars['String']>;
  token_not?: InputMaybe<Scalars['String']>;
  token_not_contains?: InputMaybe<Scalars['String']>;
  token_not_contains_nocase?: InputMaybe<Scalars['String']>;
  token_not_ends_with?: InputMaybe<Scalars['String']>;
  token_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  token_not_in?: InputMaybe<Array<Scalars['String']>>;
  token_not_starts_with?: InputMaybe<Scalars['String']>;
  token_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  token_starts_with?: InputMaybe<Scalars['String']>;
  token_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction?: InputMaybe<Scalars['String']>;
  transaction_?: InputMaybe<Transaction_Filter>;
  transaction_contains?: InputMaybe<Scalars['String']>;
  transaction_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_ends_with?: InputMaybe<Scalars['String']>;
  transaction_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_gt?: InputMaybe<Scalars['String']>;
  transaction_gte?: InputMaybe<Scalars['String']>;
  transaction_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_lt?: InputMaybe<Scalars['String']>;
  transaction_lte?: InputMaybe<Scalars['String']>;
  transaction_not?: InputMaybe<Scalars['String']>;
  transaction_not_contains?: InputMaybe<Scalars['String']>;
  transaction_not_contains_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with?: InputMaybe<Scalars['String']>;
  transaction_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_not_in?: InputMaybe<Array<Scalars['String']>>;
  transaction_not_starts_with?: InputMaybe<Scalars['String']>;
  transaction_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  transaction_starts_with?: InputMaybe<Scalars['String']>;
  transaction_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault?: InputMaybe<Scalars['String']>;
  vaultId?: InputMaybe<Scalars['BigInt']>;
  vaultId_gt?: InputMaybe<Scalars['BigInt']>;
  vaultId_gte?: InputMaybe<Scalars['BigInt']>;
  vaultId_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultId_lt?: InputMaybe<Scalars['BigInt']>;
  vaultId_lte?: InputMaybe<Scalars['BigInt']>;
  vaultId_not?: InputMaybe<Scalars['BigInt']>;
  vaultId_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vault_?: InputMaybe<Vault_Filter>;
  vault_contains?: InputMaybe<Scalars['String']>;
  vault_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_ends_with?: InputMaybe<Scalars['String']>;
  vault_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_gt?: InputMaybe<Scalars['String']>;
  vault_gte?: InputMaybe<Scalars['String']>;
  vault_in?: InputMaybe<Array<Scalars['String']>>;
  vault_lt?: InputMaybe<Scalars['String']>;
  vault_lte?: InputMaybe<Scalars['String']>;
  vault_not?: InputMaybe<Scalars['String']>;
  vault_not_contains?: InputMaybe<Scalars['String']>;
  vault_not_contains_nocase?: InputMaybe<Scalars['String']>;
  vault_not_ends_with?: InputMaybe<Scalars['String']>;
  vault_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  vault_not_in?: InputMaybe<Array<Scalars['String']>>;
  vault_not_starts_with?: InputMaybe<Scalars['String']>;
  vault_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  vault_starts_with?: InputMaybe<Scalars['String']>;
  vault_starts_with_nocase?: InputMaybe<Scalars['String']>;
};

export type VaultWithdraw_OrderBy =
  | 'amount'
  | 'amountDisplay'
  | 'emitter'
  | 'emitter__id'
  | 'id'
  | 'requestedAmount'
  | 'requestedAmountDisplay'
  | 'sender'
  | 'sender__id'
  | 'timestamp'
  | 'token'
  | 'tokenVault'
  | 'tokenVault__balance'
  | 'tokenVault__balanceDisplay'
  | 'tokenVault__id'
  | 'tokenVault__vaultId'
  | 'token__decimals'
  | 'token__id'
  | 'token__name'
  | 'token__symbol'
  | 'token__totalSupply'
  | 'token__totalSupplyDisplay'
  | 'transaction'
  | 'transaction__blockNumber'
  | 'transaction__id'
  | 'transaction__timestamp'
  | 'vault'
  | 'vaultId'
  | 'vault__id'
  | 'vault__vaultId';

export type Vault_Filter = {
  /** Filter for the block changed event. */
  _change_block?: InputMaybe<BlockChangedFilter>;
  and?: InputMaybe<Array<InputMaybe<Vault_Filter>>>;
  deposits_?: InputMaybe<VaultDeposit_Filter>;
  id?: InputMaybe<Scalars['ID']>;
  id_gt?: InputMaybe<Scalars['ID']>;
  id_gte?: InputMaybe<Scalars['ID']>;
  id_in?: InputMaybe<Array<Scalars['ID']>>;
  id_lt?: InputMaybe<Scalars['ID']>;
  id_lte?: InputMaybe<Scalars['ID']>;
  id_not?: InputMaybe<Scalars['ID']>;
  id_not_in?: InputMaybe<Array<Scalars['ID']>>;
  or?: InputMaybe<Array<InputMaybe<Vault_Filter>>>;
  owner?: InputMaybe<Scalars['String']>;
  owner_?: InputMaybe<Account_Filter>;
  owner_contains?: InputMaybe<Scalars['String']>;
  owner_contains_nocase?: InputMaybe<Scalars['String']>;
  owner_ends_with?: InputMaybe<Scalars['String']>;
  owner_ends_with_nocase?: InputMaybe<Scalars['String']>;
  owner_gt?: InputMaybe<Scalars['String']>;
  owner_gte?: InputMaybe<Scalars['String']>;
  owner_in?: InputMaybe<Array<Scalars['String']>>;
  owner_lt?: InputMaybe<Scalars['String']>;
  owner_lte?: InputMaybe<Scalars['String']>;
  owner_not?: InputMaybe<Scalars['String']>;
  owner_not_contains?: InputMaybe<Scalars['String']>;
  owner_not_contains_nocase?: InputMaybe<Scalars['String']>;
  owner_not_ends_with?: InputMaybe<Scalars['String']>;
  owner_not_ends_with_nocase?: InputMaybe<Scalars['String']>;
  owner_not_in?: InputMaybe<Array<Scalars['String']>>;
  owner_not_starts_with?: InputMaybe<Scalars['String']>;
  owner_not_starts_with_nocase?: InputMaybe<Scalars['String']>;
  owner_starts_with?: InputMaybe<Scalars['String']>;
  owner_starts_with_nocase?: InputMaybe<Scalars['String']>;
  tokenVaults_?: InputMaybe<TokenVault_Filter>;
  vaultId?: InputMaybe<Scalars['BigInt']>;
  vaultId_gt?: InputMaybe<Scalars['BigInt']>;
  vaultId_gte?: InputMaybe<Scalars['BigInt']>;
  vaultId_in?: InputMaybe<Array<Scalars['BigInt']>>;
  vaultId_lt?: InputMaybe<Scalars['BigInt']>;
  vaultId_lte?: InputMaybe<Scalars['BigInt']>;
  vaultId_not?: InputMaybe<Scalars['BigInt']>;
  vaultId_not_in?: InputMaybe<Array<Scalars['BigInt']>>;
  withdraws_?: InputMaybe<VaultWithdraw_Filter>;
};

export type Vault_OrderBy =
  | 'deposits'
  | 'id'
  | 'owner'
  | 'owner__id'
  | 'tokenVaults'
  | 'vaultId'
  | 'withdraws';

export type _Block_ = {
  __typename?: '_Block_';
  /** The hash of the block */
  hash?: Maybe<Scalars['Bytes']>;
  /** The block number */
  number: Scalars['Int'];
  /** Integer representation of the timestamp stored in blocks for the chain */
  timestamp?: Maybe<Scalars['Int']>;
};

/** The type for the top-level _meta field */
export type _Meta_ = {
  __typename?: '_Meta_';
  /**
   * Information about a specific subgraph block. The hash of the block
   * will be null if the _meta field has a block constraint that asks for
   * a block number. It will be filled if the _meta field has no block constraint
   * and therefore asks for the latest  block
   *
   */
  block: _Block_;
  /** The deployment ID */
  deployment: Scalars['String'];
  /** If `true`, the subgraph encountered indexing errors at some past block */
  hasIndexingErrors: Scalars['Boolean'];
};

export type _SubgraphErrorPolicy_ =
  /** Data will be returned even if the subgraph has indexing errors */
  | 'allow'
  /** If the subgraph has indexing errors, data will be omitted. The default. */
  | 'deny';

export type OrdersQueryQueryVariables = Exact<{
  filters?: InputMaybe<Order_Filter>;
}>;


export type OrdersQueryQuery = { __typename?: 'Query', orders: Array<{ __typename?: 'Order', id: string, orderHash: any, orderJSONString: string, orderActive: boolean, timestamp: any, expression: any, owner: { __typename?: 'Account', id: any }, validInputs?: Array<{ __typename?: 'IO', vaultId: any, token: { __typename?: 'ERC20', id: string }, tokenVault: { __typename?: 'TokenVault', id: string, balance: any, balanceDisplay: any, token: { __typename?: 'ERC20', name: string, decimals: number, symbol: string } } }> | null, validOutputs?: Array<{ __typename?: 'IO', vaultId: any, token: { __typename?: 'ERC20', id: string }, tokenVault: { __typename?: 'TokenVault', id: string, balance: any, balanceDisplay: any, token: { __typename?: 'ERC20', name: string, decimals: number, symbol: string } } }> | null, takeOrders?: Array<{ __typename?: 'TakeOrderEntity', outputIOIndex: any, inputIOIndex: any, input: any, output: any, inputDisplay: any, outputDisplay: any, timestamp: any, id: string, inputToken: { __typename?: 'ERC20', decimals: number, id: string, name: string, symbol: string }, outputToken: { __typename?: 'ERC20', decimals: number, id: string, name: string, symbol: string }, sender: { __typename?: 'Account', id: any }, transaction: { __typename?: 'Transaction', blockNumber: any, timestamp: any, id: string } }> | null }> };

export type TakeOrderEntitiesDynamicFilterQueryVariables = Exact<{
  filters?: InputMaybe<TakeOrderEntity_Filter>;
}>;


export type TakeOrderEntitiesDynamicFilterQuery = { __typename?: 'Query', takeOrderEntities: Array<{ __typename?: 'TakeOrderEntity', id: string, input: any, inputDisplay: any, output: any, outputDisplay: any, timestamp: any, order: { __typename?: 'Order', orderHash: any, id: string, owner: { __typename?: 'Account', id: any } }, inputToken: { __typename?: 'ERC20', id: string, name: string, symbol: string, decimals: number }, outputToken: { __typename?: 'ERC20', id: string, name: string, symbol: string, decimals: number }, sender: { __typename?: 'Account', id: any }, transaction: { __typename?: 'Transaction', timestamp: any, id: string } }> };

export type TokenVaultsQueryVariables = Exact<{
  filters?: InputMaybe<TokenVault_Filter>;
}>;


export type TokenVaultsQuery = { __typename?: 'Query', tokenVaults: Array<{ __typename?: 'TokenVault', vaultId: any, balance: any, balanceDisplay: any, id: string, orders?: Array<{ __typename?: 'Order', id: string, orderHash: any, orderActive: boolean, expression: any, expressionDeployer: any }> | null, owner: { __typename?: 'Account', id: any }, token: { __typename?: 'ERC20', symbol: string, name: string, decimals: number, id: string } }> };


export const OrdersQueryDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"ordersQuery"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"Order_filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"orders"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"where"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}},{"kind":"Argument","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"EnumValue","value":"timestamp"}},{"kind":"Argument","name":{"kind":"Name","value":"orderDirection"},"value":{"kind":"EnumValue","value":"desc"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"orderHash"}},{"kind":"Field","name":{"kind":"Name","value":"owner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"orderJSONString"}},{"kind":"Field","name":{"kind":"Name","value":"orderActive"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"expression"}},{"kind":"Field","name":{"kind":"Name","value":"validInputs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"vaultId"}},{"kind":"Field","name":{"kind":"Name","value":"token"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"tokenVault"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"balance"}},{"kind":"Field","name":{"kind":"Name","value":"balanceDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"token"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"decimals"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"validOutputs"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"vaultId"}},{"kind":"Field","name":{"kind":"Name","value":"token"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"tokenVault"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"balance"}},{"kind":"Field","name":{"kind":"Name","value":"balanceDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"token"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"decimals"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"takeOrders"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"outputIOIndex"}},{"kind":"Field","name":{"kind":"Name","value":"inputIOIndex"}},{"kind":"Field","name":{"kind":"Name","value":"input"}},{"kind":"Field","name":{"kind":"Name","value":"output"}},{"kind":"Field","name":{"kind":"Name","value":"inputDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"outputDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"inputToken"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"decimals"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}},{"kind":"Field","name":{"kind":"Name","value":"outputToken"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"decimals"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}}]}},{"kind":"Field","name":{"kind":"Name","value":"sender"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"transaction"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"blockNumber"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<OrdersQueryQuery, OrdersQueryQueryVariables>;
export const TakeOrderEntitiesDynamicFilterDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"takeOrderEntitiesDynamicFilter"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"TakeOrderEntity_filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"takeOrderEntities"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"where"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}},{"kind":"Argument","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"EnumValue","value":"timestamp"}},{"kind":"Argument","name":{"kind":"Name","value":"orderDirection"},"value":{"kind":"EnumValue","value":"desc"}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"input"}},{"kind":"Field","name":{"kind":"Name","value":"inputDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"output"}},{"kind":"Field","name":{"kind":"Name","value":"outputDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"order"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"orderHash"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"owner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"inputToken"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"decimals"}}]}},{"kind":"Field","name":{"kind":"Name","value":"outputToken"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"decimals"}}]}},{"kind":"Field","name":{"kind":"Name","value":"sender"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"transaction"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"timestamp"}},{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<TakeOrderEntitiesDynamicFilterQuery, TakeOrderEntitiesDynamicFilterQueryVariables>;
export const TokenVaultsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"tokenVaults"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"TokenVault_filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"tokenVaults"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"where"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"vaultId"}},{"kind":"Field","name":{"kind":"Name","value":"orders"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"orderHash"}},{"kind":"Field","name":{"kind":"Name","value":"orderActive"}},{"kind":"Field","name":{"kind":"Name","value":"expression"}},{"kind":"Field","name":{"kind":"Name","value":"expressionDeployer"}}]}},{"kind":"Field","name":{"kind":"Name","value":"owner"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}},{"kind":"Field","name":{"kind":"Name","value":"balance"}},{"kind":"Field","name":{"kind":"Name","value":"balanceDisplay"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"token"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"symbol"}},{"kind":"Field","name":{"kind":"Name","value":"name"}},{"kind":"Field","name":{"kind":"Name","value":"decimals"}},{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]}}]} as unknown as DocumentNode<TokenVaultsQuery, TokenVaultsQueryVariables>;