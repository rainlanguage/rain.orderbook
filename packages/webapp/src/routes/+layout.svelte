<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { supportedChainsList } from '$lib/chains';
	import { defaultWagmiConfig } from '$lib/stores/wagmi';
	import { injected, walletConnect } from '@wagmi/connectors';
	import { type Chain } from '@wagmi/core/chains';
	import { PUBLIC_WALLETCONNECT_PROJECT_ID } from '$env/static/public';
	import { page } from '$app/stores';
	import Homepage from '$lib/components/Homepage.svelte';
	import LoadingWrapper from '$lib/components/LoadingWrapper.svelte';

	// Query client for caching
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	const initWallet = async () => {
		const erckit = defaultWagmiConfig({
			supportedChains: supportedChainsList as unknown as Chain[],
			appName: 'Rain Language',
			connectors: [injected(), walletConnect({ projectId: PUBLIC_WALLETCONNECT_PROJECT_ID })],
			chains: supportedChainsList as unknown as Chain[],
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		await erckit.initWagmi();
	};

	$: if (browser && window.navigator) {
		initWallet();
	}
</script>

<QueryClientProvider client={queryClient}>
	<LoadingWrapper>
		{#if $page.url.pathname === '/'}
			<Homepage {colorTheme} />
		{:else}
			<div
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
