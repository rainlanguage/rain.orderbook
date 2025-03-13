import type { Config, CreateConnectorFn, GetAccountReturnType } from '@wagmi/core';
import type { Chain } from '@wagmi/core/chains';
import type { AppKit } from '@reown/appkit';
import type { Writable } from 'svelte/store';

// Store types
export type WagmiStores = {
  connected: Writable<boolean>;
  wagmiLoaded: Writable<boolean>;
  chainId: Writable<number | null | undefined>;
  signerAddress: Writable<string | null>;
  configuredConnectors: Writable<CreateConnectorFn[]>;
  loading: Writable<boolean>;
  appKitModal: Writable<AppKit>;
  wagmiConfig: Writable<Config>;
};

// Configuration types
export interface DefaultConfigProps {
  appName: string;
  appIcon?: string | null;
  appDescription?: string | null;
  appUrl?: string | null;
  autoConnect?: boolean;
  alchemyId?: string | null;
  chains?: Chain[] | null;
  connectors: CreateConnectorFn[];
  projectId: string;
}

export interface ConfigResult {
  init: () => Promise<void>;
}

// Function types
export type DefaultConfigFn = (props: DefaultConfigProps) => ConfigResult;
export type InitFn = () => Promise<void>;
export type WCFn = () => Promise<{ success: boolean }>;
export type DisconnectWagmiFn = () => Promise<void>;
export type WaitForAccountFn = () => Promise<GetAccountReturnType>;
export type WaitForConnectionFn = () => Promise<GetAccountReturnType>;
export type SetupListenersFn = () => void;
export type HandleAccountChangeFn = (data: GetAccountReturnType) => Promise<void>;

// Simplified context type for components that only need core functionality
export type WagmiContext = {
  connected: Writable<boolean>;
  signerAddress: Writable<string | null>;
  wagmiConfig: Writable<Config>;
  appKitModal: Writable<AppKit>;
  chainId: Writable<number | null | undefined>;
};