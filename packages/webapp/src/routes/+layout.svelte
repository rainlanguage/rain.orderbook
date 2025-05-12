<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { supportedChainsList } from '$lib/chains';
	import { defaultConfig } from '$lib/stores/wagmi';
	import { injected, walletConnect } from '@wagmi/connectors';
	import { type Chain } from '@wagmi/core/chains';
	import { PUBLIC_WALLETCONNECT_PROJECT_ID } from '$env/static/public';
	import { page } from '$app/stores';
	import Homepage from '$lib/components/Homepage.svelte';
	import LoadingWrapper from '$lib/components/LoadingWrapper.svelte';
	import { ToastProvider, WalletProvider } from '@rainlanguage/ui-components';
	import { signerAddress } from '$lib/stores/wagmi';
	import { toasts } from '$lib/stores/toasts';
	import { settings as cachedSettings } from '$lib/stores/settings';

	const { settings } = $page.data.stores;
	cachedSettings.set(settings);

	// Query client for caching
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	let walletInitError: string | null = null;

	const initWallet = async () => {
		try {
			const erckit = defaultConfig({
				appName: 'Rain Language',
				connectors: [injected(), walletConnect({ projectId: PUBLIC_WALLETCONNECT_PROJECT_ID })],
				chains: supportedChainsList as unknown as Chain[],
				projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
			});
			await erckit.init();
			walletInitError = null;
		} catch (error) {
			walletInitError = `Failed to initialize wallet connection: ${error instanceof Error ? error.message : 'Unknown error'}. Please try again or check console.`;
		}
	};

	$: if (browser && window.navigator) {
		initWallet();
	}
</script>

{#if walletInitError}
	<div
		class="fixed bottom-4 left-1/2 z-[100] -translate-x-1/2 transform rounded-lg bg-red-500 px-6 py-3 text-white shadow-md"
	>
		{walletInitError}
	</div>
{/if}

<ToastProvider {toasts}>
	<WalletProvider account={signerAddress}>
		<QueryClientProvider client={queryClient}>
			<LoadingWrapper>
				{#if $page.url.pathname === '/'}
					<Homepage {colorTheme} />
				{:else}
					<div
						data-testid="layout-container"
						class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400"
					>
						<Sidebar {colorTheme} page={$page} />
						<main class="mx-auto h-full w-full grow overflow-x-auto px-4 pt-14 lg:ml-64 lg:p-8">
							<slot />
						</main>
					</div>
				{/if}
			</LoadingWrapper>
		</QueryClientProvider>
	</WalletProvider>
</ToastProvider>
