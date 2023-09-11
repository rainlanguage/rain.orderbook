<script lang="ts">
	import '../app.postcss';
	import { getOrderbookAddress, initOrderbook, orderbook, subgraphClient } from '$lib';
	import { browser } from '$app/environment';
	import { configureChains, getWalletClient } from '@wagmi/core';
	import { avalanche, mainnet, goerli, polygonMumbai } from '@wagmi/core/chains';
	import { createConfig, account, walletClient, network } from 'svelte-wagmi-stores';
	import { Web3Modal } from '@web3modal/html';
	import { EthereumClient, w3mConnectors, w3mProvider } from '@web3modal/ethereum';

	import {
		Button,
		Dropdown,
		DropdownDivider,
		DropdownItem,
		FloatingLabelInput,
		Input,
		Label,
		NavBrand,
		NavHamburger,
		NavLi,
		NavUl,
		Navbar,
		Sidebar,
		SidebarGroup,
		SidebarItem,
		SidebarWrapper
	} from 'flowbite-svelte';
	import { onMount } from 'svelte';

	let address: string = '0x34200e026fbac0c902a0ff18e77a49265ca6ac99',
		subgraphEndpoint: string =
			'https://api.thegraph.com/subgraphs/name/hardyjosh/orderbook-0xd14c2ba8779c6c4fba';

	$: if (subgraphEndpoint) {
		getOrderbookAddress(subgraphEndpoint).then(({ orderbookAddress }) => {
			if (orderbookAddress) address = orderbookAddress;
		});
	}

	onMount(async () => {
		subgraphEndpoint = localStorage.getItem('subgraphEndpoint') || subgraphEndpoint;
		if (subgraphEndpoint) {
			getOrderbookAddress(subgraphEndpoint).then(({ orderbookAddress }) => {
				if (orderbookAddress) address = orderbookAddress;
				initOrderbook({ address, subgraphEndpoint });
			});
		}
	});

	// all this boilerplate is from the web3modal docs
	const chains = [mainnet, avalanche, goerli, polygonMumbai];
	const projectId = import.meta.env.VITE_PROJECT_ID;

	const { publicClient } = configureChains(chains, [w3mProvider({ projectId })]);

	// except here we're using createConfig form this package instead of wagmi
	const wagmiConfig = createConfig({
		autoConnect: true,
		connectors: w3mConnectors({ projectId, chains }),
		publicClient
	});

	const ethereumClient = new EthereumClient(wagmiConfig, chains);

	let web3modal: Web3Modal;

	// necessary if you're using SSR, because there's no window for the modal to attach to
	$: if (browser) {
		web3modal = new Web3Modal(
			{
				projectId,
				themeVariables: {
					'--w3m-logo-image-url':
						'https://uploads-ssl.webflow.com/627b9589504d3adc8db27d80/6283b4807a13aa6e4df650ee_RainProtocol_Logo%201.svg',
					'--w3m-background-color': '#000000'
				}
			},
			ethereumClient
		);
		web3modal.setDefaultChain(goerli);
	}
</script>

<div class="flex flex-col h-screen relative w-screen">
	<Navbar
		let:hidden
		let:toggle
		class="px-2 sm:px-4 py-2.5 sticky w-full z-20 top-0 left-0 border-b"
		fluid
	>
		<NavBrand href="/">
			<span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">
				Orderbook
			</span>
		</NavBrand>
		<NavHamburger on:click={toggle} />
		<Button
			outline
			on:click={() => {
				web3modal.openModal();
			}}
		>
			{#if !$account?.isConnected}
				Connect wallet
			{:else}
				Connected
			{/if}
		</Button>
	</Navbar>
	<div class="flex overflow-scroll relative grow overflow-x-clip">
		<Sidebar class="sticky top-0 bottom-0 shrink-0 ">
			<SidebarWrapper class="h-full bg-white border-r-gray-200 border-r rounded-none">
				<SidebarGroup>
					<SidebarItem label="Home" href="/" />
					<SidebarItem label="Orders" href="/orders" />
					<SidebarItem label="Vaults" href="/vaults" />
					<SidebarItem label="Report" href="/queries/take-orders" />
				</SidebarGroup>
				<SidebarGroup border>
					<div class="flex flex-col gap-y-4">
						<FloatingLabelInput
							style="outlined"
							label="Subgraph endpoint"
							type="text"
							id="subgraphEndpoint"
							bind:value={subgraphEndpoint}
						/>
						<FloatingLabelInput
							style="outlined"
							label="Orderbook address"
							type="text"
							id="address"
							bind:value={address}
						/>
						<Button
							on:click={() => {
								initOrderbook({ address, subgraphEndpoint });
								localStorage.setItem('subgraphEndpoint', subgraphEndpoint);
							}}>Save</Button
						>
					</div>
				</SidebarGroup>
			</SidebarWrapper>
		</Sidebar>
		{#if $subgraphClient}
			<div class="grow p-4 overflow-x-hidden">
				<slot />
			</div>
		{/if}
	</div>
</div>
