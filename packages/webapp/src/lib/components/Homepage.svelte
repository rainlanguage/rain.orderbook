<script lang="ts">
	import { logoDark, logoLight } from '@rainlanguage/ui-components';
	import { Button } from 'flowbite-svelte';
	import { goto } from '$app/navigation';

	export let colorTheme;
</script>

<div data-testid="homepage" class="mx-auto flex min-h-screen w-full flex-col">
	<div class="flex h-16 w-full items-center bg-gray-100 dark:bg-gray-800">
		<div class="flex w-full items-center justify-between px-4">
			<div class="flex h-full flex-row items-center gap-x-4">
				<img src={$colorTheme === 'light' ? logoLight : logoDark} alt="Rainlang logo" class="h-8" />
			</div>
			<div class="flex h-full flex-row items-center gap-x-2 dark:text-gray-100 md:gap-x-6">
				<a href="https://docs.rainlang.xyz/raindex/overview">Docs</a>
				<a href="/deploy">Dashboard</a>
			</div>
		</div>
	</div>
	<div
		class="flex w-full flex-row justify-center border-b-2 border-gray-300 bg-gray-200 px-4 dark:border-gray-600 dark:bg-gray-700"
	>
		<div class="my-24 flex w-full max-w-4xl flex-col">
			<h1
				class="light:text-gray-900 text-left font-instrument-serif text-7xl font-light leading-tight dark:text-white lg:text-8xl"
			>
				Trade onchain
				<br />
				without limits
			</h1>
			<div class="mt-8 flex justify-start gap-x-4">
				<Button color="blue" size="lg" on:click={() => goto('/deploy')}>Deploy a strategy</Button>
				<a href="https://docs.rainlang.xyz/raindex/overview">
					<Button color="light" size="lg">Learn</Button>
				</a>
			</div>
		</div>
	</div>
	<div
		class="flex w-full flex-grow justify-center bg-gray-100 px-4 dark:bg-gray-800 dark:text-white"
	>
		<div class="mx-auto mt-12 grid max-w-4xl grid-cols-1 gap-16 md:grid-cols-3">
			<div class="flex flex-col gap-y-3">
				<h2 class="text-xl font-semibold">Automated</h2>
				<p>No more manual trades. Raindex brings automated trading strategies directly onchain.</p>
			</div>

			<div class="flex flex-col gap-y-3">
				<h2 class="text-xl font-semibold">Trustless</h2>
				<p>
					Raindex is non-custodial – your assets remain in your control always. Strategies are
					trustlessly executed by competing solvers.
				</p>
			</div>

			<div class="flex flex-col gap-y-3">
				<h2 class="text-xl font-semibold">Infinitely flexible</h2>
				<p>
					Deploy existing strategies in minutes or write your own using Rainlang. DCA, limit orders,
					liquidity provision and more.
				</p>
			</div>
		</div>
	</div>
</div>
