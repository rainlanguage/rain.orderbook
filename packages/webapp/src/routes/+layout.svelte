<script lang="ts">
	import '../app.css';
	import { QueryClient, QueryClientProvider } from '@tanstack/svelte-query';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { colorTheme } from '$lib/darkMode';
	import { browser } from '$app/environment';
	import { supportedChainsList } from '$lib/chains';
	import { defaultConfig } from '$lib/stores/wagmi';
	import { injected } from '@wagmi/connectors';
	import { type Chain } from '@wagmi/core/chains';
	import { PUBLIC_WALLETCONNECT_PROJECT_ID } from '$env/static/public';
	import { Progressbar } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import Homepage from '$lib/components/Homepage.svelte';
	import { beforeNavigate, afterNavigate } from '$app/navigation';
	import { isNavigating } from '$lib/stores/loading';
	import { tick } from 'svelte';

	let progress = 0;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let interval: any;

	// Gradually increase progress while navigating
	const startLoading = () => {
		isNavigating.set(true);
		progress = 10; // Start at 10% so it's visible

		clearInterval(interval);
		interval = setInterval(() => {
			if (progress < 90) {
				progress += 5;
			}
		}, 300);
	};

	// Stop loading and reset progress
	const stopLoading = async () => {
		clearInterval(interval);
		progress = 100;
		await tick();
		setTimeout(() => {
			progress = 0; // Reset for next navigation
			isNavigating.set(false);
		}, 500);
	};

	// Track navigation state
	beforeNavigate(() => startLoading());
	afterNavigate(() => stopLoading());

	// Query client for caching
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
			projectId: PUBLIC_WALLETCONNECT_PROJECT_ID
		});
		await erckit.init();
	};

	$: if (browser && window.navigator) {
		initWallet();
	}
</script>

<QueryClientProvider client={queryClient}>
	{#if $isNavigating}
		<div class="fixed left-0 top-0 z-50 w-full">
			<Progressbar {progress} color="blue" animate size="h-1" />
		</div>
	{/if}

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
</QueryClientProvider>
