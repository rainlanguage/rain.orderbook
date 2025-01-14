import type {
    AppStoresInterface,
    ConfigSource,
    OrderbookConfigSource,
    OrderbookRef
} from '@rainlanguage/ui-components';
import { writable, derived } from 'svelte/store';
import settingsJson from '$lib/settings-12-11-24.json';
import pkg from 'lodash';
import { AppKit, createAppKit } from '@reown/appkit';
import { mainnet, arbitrum, flare } from '@reown/appkit/networks';
import { WagmiAdapter } from '@reown/appkit-adapter-wagmi';
import { browser } from '$app/environment';
const { pickBy } = pkg;

export interface LayoutData {
    stores: AppStoresInterface;
}

export const load = () => {
	let appKitModal: AppKit | undefined;
    // Initialize Reown AppKit (only in browser)
    if (browser) {
        const projectId = 'a68d9b4020ecec5fd5d32dcd4008e7f4'; // Replace with your actual project ID

        const networks = [mainnet, arbitrum];

        const wagmiAdapter = new WagmiAdapter({
            projectId,
            networks
        });

        const metadata = {
            name: 'Your App Name',
            description: 'Your App Description',
            url: 'http://localhost:5173', // Update with your domain
            icons: [] // Update with your icon
        };

        const modal = createAppKit({
            adapters: [wagmiAdapter],
            networks: [mainnet, arbitrum, flare],
            metadata,
            projectId,
            features: {
                analytics: true
            }
        });

        // Export the modal instance if needed elsewhere
        appKitModal = modal;
    }

    const activeNetworkRef = writable<string>('');
    const settings = writable<ConfigSource | undefined>(settingsJson);
    const activeOrderbookRef = writable<string>('');
    const activeOrderbook = derived(
        [settings, activeOrderbookRef],
        ([$settings, $activeOrderbookRef]) =>
            $settings?.orderbooks !== undefined && $activeOrderbookRef !== undefined
                ? $settings.orderbooks[$activeOrderbookRef]
                : undefined
    );
    const subgraphUrl = derived([settings, activeOrderbook], ([$settings, $activeOrderbook]) =>
        $settings?.subgraphs !== undefined && $activeOrderbook?.subgraph !== undefined
            ? $settings.subgraphs[$activeOrderbook.subgraph]
            : undefined
    );
    const activeNetworkOrderbooks = derived(
        [settings, activeNetworkRef],
        ([$settings, $activeNetworkRef]) =>
            $settings?.orderbooks
                ? (pickBy(
                    $settings.orderbooks,
                    (orderbook) => orderbook.network === $activeNetworkRef
                ) as Record<OrderbookRef, OrderbookConfigSource>)
                : ({} as Record<OrderbookRef, OrderbookConfigSource>)
    );

    const accounts = derived(settings, ($settings) => $settings?.accounts);
    const activeAccountsItems = writable<Record<string, string>>({});

    const activeAccounts = derived(
        [accounts, activeAccountsItems],
        ([$accounts, $activeAccountsItems]) =>
            Object.keys($activeAccountsItems).length === 0
                ? {}
                : Object.fromEntries(
                    Object.entries($accounts || {}).filter(([key]) => key in $activeAccountsItems)
                )
    );

    return {
        stores: {
            settings,
            activeSubgraphs: writable<Record<string, string>>({}),
            accounts,
            activeAccountsItems,
            activeAccounts,
            activeOrderStatus: writable<boolean | undefined>(undefined),
            orderHash: writable<string>(''),
            hideZeroBalanceVaults: writable<boolean>(false),
            activeNetworkRef,
            activeOrderbookRef,
            activeOrderbook,
            subgraphUrl,
            activeNetworkOrderbooks,
        },
		appKitModal: browser ? writable(appKitModal) : writable(null)
    };
};

export const ssr = false;