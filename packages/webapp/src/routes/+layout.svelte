<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { supportedChainsList } from '$lib/chains';
	import { defaultConfig } from '$lib/stores/wagmi';
	import { PUBLIC_WALLETCONNECT_ID } from '$env/static/public';
	import { injected } from '@wagmi/connectors';
	import { type Chain } from '@wagmi/core/chains';

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});

	const initWallet = async () => {
		const erckit = defaultConfig({
			appName: 'Rain Language',
			connectors: [injected()],
			chains: supportedChainsList as unknown as Chain[],
			projectId: PUBLIC_WALLETCONNECT_ID
		});
		await erckit.init();
	};

	$: if (browser && window.navigator) {
		initWallet();
	}
</script>

<QueryClientProvider client={queryClient}>
	<div class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400">
		<Sidebar {colorTheme} />
		<main class="ml-64 h-full w-full grow overflow-x-auto p-8">
			<slot />
		</main>
	</div>
</QueryClientProvider>
