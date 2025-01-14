<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import { SidebarWebapp } from '@rainlanguage/ui-components';
	import { colorTheme } from '$lib/darkMode';
	import { defaultConfig } from 'svelte-wagmi';
	import { injected, walletConnect } from '@wagmi/connectors';
	import { flare } from '@wagmi/core/chains';
	import { browser } from '$app/environment';

	const initWallet = async () => {
		const erckit = defaultConfig({
			autoConnect: true,
			appName: 'cyclo',
			walletConnectProjectId: 'a68d9b4020ecec5fd5d32dcd4008e7f4',
			chains: [flare],
			connectors: [injected(), walletConnect({ projectId: 'a68d9b4020ecec5fd5d32dcd4008e7f4' })]
		});
		await erckit.init();
	};

	$: if (browser && window.navigator) {
		initWallet();
	}

	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				staleTime: Infinity
			}
		}
	});
</script>

<QueryClientProvider client={queryClient}>
	<div class="flex min-h-screen w-full justify-start bg-white dark:bg-gray-900 dark:text-gray-400">
		<SidebarWebapp {colorTheme} />
		<main class="ml-64 h-full w-full grow overflow-x-auto p-8">
			<slot />
		</main>
	</div>
</QueryClientProvider>
