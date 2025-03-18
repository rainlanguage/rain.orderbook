import { type CreateConnectorFn, type Config } from '@wagmi/core';
import { type Chain } from '@wagmi/core/chains';
import { AppKit } from '@reown/appkit';
export declare const supportedChains: Chain[];
export declare const connected: import('svelte/store').Writable<boolean>;
export declare const wagmiLoaded: import('svelte/store').Writable<boolean>;
export declare const chainId: import('svelte/store').Writable<number | null | undefined>;
export declare const signerAddress: import('svelte/store').Writable<string | null>;
export declare const configuredConnectors: import('svelte/store').Writable<CreateConnectorFn[]>;
export declare const loading: import('svelte/store').Writable<boolean>;
export declare const appKitModal: import('svelte/store').Writable<AppKit>;
export declare const wagmiConfig: import('svelte/store').Writable<Config>;
type DefaultConfigProps = {
	appName: string;
	appIcon?: string | null;
	appDescription?: string | null;
	appUrl?: string | null;
	autoConnect?: boolean;
	alchemyId?: string | null;
	chains?: Chain[] | null;
	connectors: CreateConnectorFn[];
	projectId: string;
	supportedChains: Chain[];
};
export declare const defaultWagmiConfig: ({
	appName,
	appDescription,
	appUrl,
	appIcon,
	autoConnect,
	chains,
	connectors,
	projectId,
	supportedChains
}: DefaultConfigProps) => {
	initWagmi: () => Promise<void>;
};
export declare const initWagmi: () => Promise<void>;
export declare const WC: () => Promise<{
	success: boolean;
}>;
export declare const disconnectWagmi: () => Promise<void>;
export {};
