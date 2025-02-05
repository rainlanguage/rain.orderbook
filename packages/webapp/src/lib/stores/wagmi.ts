import { writable, get } from 'svelte/store';
import {
	createConfig,
	getAccount,
	disconnect,
	watchAccount,
	reconnect,
	type CreateConnectorFn,
	type GetAccountReturnType,
	type Config,
	http
} from '@wagmi/core';
import { type Chain } from '@wagmi/core/chains';
import { AppKit, createAppKit } from '@reown/appkit';
import { WagmiAdapter } from '@reown/appkit-adapter-wagmi';
import { supportedChainsList } from '$lib/chains';

export const connected = writable<boolean>(false);
export const wagmiLoaded = writable<boolean>(false);
export const chainId = writable<number | null | undefined>(null);
export const signerAddress = writable<string | null>(null);
export const configuredConnectors = writable<CreateConnectorFn[]>([]);
export const loading = writable<boolean>(true);
export const appKitModal = writable<AppKit>();
export const wagmiConfig = writable<Config>();

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
};

export const defaultConfig = ({
	appName,
	appDescription = null,
	appUrl = null,
	appIcon = null,
	autoConnect = true,
	chains = [],
	connectors,
	projectId
}: DefaultConfigProps) => {
	if (connectors) configuredConnectors.set(connectors);

	const url = http();

	const chainsToUse = chains ? chains.map((chain) => chain) : [];
	const transports = chains
		? chains.reduce(
				(acc, chain) => ({
					...acc,
					[chain.id]: url
				}),
				{}
			)
		: {};
	const config = createConfig({
		chains: [supportedChainsList[0], ...supportedChainsList.slice(1)] as [Chain, ...Chain[]],
		transports,
		connectors: get(configuredConnectors)
	});

	wagmiConfig.set(config);

	if (autoConnect) reconnect(config);

	const wagmiAdapter = new WagmiAdapter({
		projectId,
		networks: chainsToUse
	});

	const metadata = {
		name: appName,
		description: appDescription || 'AppKit Integration',
		url: appUrl || window.location.origin,
		icons: appIcon ? [appIcon] : []
	};

	const modal = createAppKit({
		adapters: [wagmiAdapter],
		networks: [supportedChainsList[0], ...supportedChainsList.slice(1)] as [Chain, ...Chain[]],
		metadata,
		projectId,
		features: {
			analytics: false,
			socials: [],
			email: false
		}
	});

	appKitModal.set(modal);
	wagmiLoaded.set(true);

	return { init };
};

export const init = async () => {
	try {
		setupListeners();
		const account = await waitForConnection();
		if (account.address) {
			const chain = get(wagmiConfig).chains.find((chain) => chain.id === account.chainId);
			if (chain) chainId.set(chain.id);
			connected.set(true);
			signerAddress.set(account.address.toLowerCase());
		}
		loading.set(false);
	} catch {
		loading.set(false);
	}
};

const setupListeners = () => {
	watchAccount(get(wagmiConfig), {
		onChange(data) {
			handleAccountChange(data);
		}
	});
};

const handleAccountChange = (data: GetAccountReturnType) => {
	return (async () => {
		if (get(wagmiLoaded) && data.address) {
			const chain = get(wagmiConfig).chains.find((chain) => chain.id === data.chainId);

			if (chain) chainId.set(chain.id);
			connected.set(true);
			loading.set(false);
			signerAddress.set(data.address.toLowerCase());
		} else if (data.isDisconnected && get(connected)) {
			loading.set(false);
			await disconnectWagmi();
		}
	})();
};

export const WC = async () => {
	try {
		get(appKitModal).open();
		await waitForAccount();

		return { success: true };
	} catch {
		return { success: false };
	}
};

export const disconnectWagmi = async () => {
	await disconnect(get(wagmiConfig));
	connected.set(false);
	chainId.set(null);
	signerAddress.set(null);
	loading.set(false);
};

const waitForAccount = () => {
	return new Promise((resolve, reject) => {
		const unsub1 = get(appKitModal).subscribeEvents((newState) => {
			if (newState.data.event === 'MODAL_CLOSE') {
				reject('modal closed');
				unsub1();
			}
		});
		const unsub = watchAccount(get(wagmiConfig), {
			onChange(data) {
				if (data?.isConnected) {
					resolve(data);
					unsub();
				}
			}
		});
	});
};

const waitForConnection = (): Promise<GetAccountReturnType> =>
	new Promise((resolve, reject) => {
		const attemptToGetAccount = () => {
			const account = getAccount(get(wagmiConfig));
			if (account.isDisconnected) reject('account is disconnected');
			if (account.isConnecting) {
				setTimeout(attemptToGetAccount, 250);
			} else {
				resolve(account);
			}
		};

		attemptToGetAccount();
	});
