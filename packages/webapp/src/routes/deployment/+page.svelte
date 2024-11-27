<script lang="ts">
	import { DropdownRadio, Checkbox } from '@rainlanguage/ui-components';
	import {
		DotrainOrderGui,
		// type AddOrderCalldataResult,
		// type AllowancesResult,
		type ApprovalCalldataResult,
		type AvailableDeployments,
		type DepositAndAddOrderCalldataResult,
		// type DepositCalldataResult,
		type GuiDeposit,
		type GuiFieldDefinition,
		type TokenInfos
	} from '@rainlanguage/orderbook/js_api';
	import { onMount } from 'svelte';
	import { Button, Input, Label } from 'flowbite-svelte';
	import { createWalletClient, custom, type Chain } from 'viem';
	import { base, flare, arbitrum, polygon, bsc, mainnet, linea } from 'viem/chains';

	const chains: Record<number, Chain> = {
		[base.id]: base,
		[flare.id]: flare,
		[arbitrum.id]: arbitrum,
		[polygon.id]: polygon,
		[bsc.id]: bsc,
		[mainnet.id]: mainnet,
		[linea.id]: linea
	};

	let dotrain = '';
	onMount(async () => {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.webapp/refs/heads/main/public/_strategies/raindex/2-dynamic-spread/dynamic-spread.rain'
		);
		dotrain = await response.text();
	});

	let gui: DotrainOrderGui | undefined = undefined;
	let availableDeployments: Record<string, { label: string }> = {};
	async function initialize() {
		try {
			let deployments: AvailableDeployments =
				await DotrainOrderGui.getAvailableDeployments(dotrain);
			availableDeployments = Object.fromEntries(
				deployments.map((deployment) => [
					deployment.deployment_name,
					{
						label: deployment.deployment_name,
						deployment
					}
				])
			);
		} catch (error) {
			// eslint-disable-next-line no-console
			console.error('Failed to load deployments:', error);
		}
	}
	$: if (dotrain) {
		initialize();
	}

	let selectedDeployment: string | undefined = undefined;
	async function handleDeploymentChange(deployment: string) {
		if (!deployment) return;

		try {
			gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment);
		} catch (error) {
			// eslint-disable-next-line no-console
			console.error('Failed to get gui:', error);
		}
	}
	$: if (selectedDeployment) {
		handleDeploymentChange(selectedDeployment as string);
	}

	let tokenInfos: TokenInfos;
	function getTokenInfos() {
		if (!gui) return;
		tokenInfos = gui.getTokenInfos();
	}

	let allFieldDefinitions: GuiFieldDefinition[] = [];
	function getAllFieldDefinitions() {
		if (!gui) return;
		allFieldDefinitions = gui.getAllFieldDefinitions();
	}

	let allDeposits: GuiDeposit[] = [];
	function getDeposits() {
		if (!gui) return;
		allDeposits = gui.getCurrentDeployment().deposits;
	}

	$: if (gui) {
		getTokenInfos();
		getAllFieldDefinitions();
		getDeposits();
	}

	export function getChainById(chainId: number): Chain {
		const chain = chains[chainId];
		if (!chain) {
			throw new Error(`Unsupported chain ID: ${chainId}`);
		}
		return chain;
	}

	async function handleAddOrder() {
		try {
			if (!gui) return;

			const walletClient = createWalletClient({
				chain: getChainById(gui.getCurrentDeployment().deployment.order.network['chain-id']),
				// @ts-expect-error window.ethereum is not typed
				transport: custom(window.ethereum!)
			});
			const [account] = await walletClient.getAddresses();

			const approvals: ApprovalCalldataResult = await gui.generateApprovalCalldatas(account);
			for (const approval of approvals) {
				await walletClient.sendTransaction({
					account,
					to: approval.token as `0x${string}`,
					data: approval.calldata as `0x${string}`
				});
			}

			const calldata: DepositAndAddOrderCalldataResult =
				await gui.generateDepositAndAddOrderCalldatas();
			await walletClient.sendTransaction({
				account,
				// @ts-expect-error orderbook is not typed
				to: gui.getCurrentDeployment().deployment.order.orderbook.address as `0x${string}`,
				data: calldata as `0x${string}`
			});
		} catch (error) {
			// eslint-disable-next-line no-console
			console.error('Failed to add order:', error);
		}
	}

	let useCustomVaultIds = false;
	let customVaultId = '';
	async function handlePopulateVaultIds() {
		try {
			if (!gui) return;

			if (useCustomVaultIds && customVaultId) {
				// Convert string to BigInt and then to hex string
				const vaultIdBigInt = BigInt(customVaultId);
				if (vaultIdBigInt < 0n) {
					console.error('Invalid vault ID - must be non-negative');
					return;
				}
				gui.populateVaultIds('0x' + vaultIdBigInt.toString(16));
			} else {
				gui.populateVaultIds();
			}

			// Trigger reactivity
			gui = gui;
		} catch (error) {
			console.error('Failed to populate vault IDs:', error);
		}
	}
</script>

<div class="mb-4">
	<Label class="mb-2 whitespace-nowrap text-xl">Deployments</Label>
	<DropdownRadio options={availableDeployments} bind:value={selectedDeployment}>
		<svelte:fragment slot="content" let:selectedOption let:selectedRef>
			{#if selectedRef === undefined}
				<span>Select a deployment</span>
			{:else if selectedOption?.label}
				<span>{selectedOption.label}</span>
			{:else}
				<span>{selectedRef}</span>
			{/if}
		</svelte:fragment>

		<svelte:fragment slot="option" let:option let:ref>
			<div class="w-full overflow-hidden overflow-ellipsis">
				<div class="text-md break-word">{option.label ? option.label : ref}</div>
			</div>
		</svelte:fragment>
	</DropdownRadio>
</div>

{#if allFieldDefinitions.length > 0}
	<Label class="my-4 whitespace-nowrap text-2xl underline">Field Values</Label>

	{#each allFieldDefinitions as fieldDefinition}
		<div class="mb-4 flex flex-col gap-2">
			<Label class="whitespace-nowrap text-xl">{fieldDefinition.name}</Label>

			<DropdownRadio
				options={{
					...Object.fromEntries(
						(fieldDefinition.presets ?? []).map((preset) => [
							preset.id,
							{
								label: preset.name,
								id: preset.id
							}
						])
					),
					...{ custom: { label: 'Custom value', id: '' } }
				}}
				on:change={({ detail }) => {
					gui?.saveFieldValue(fieldDefinition.binding, {
						isPreset: detail.value !== 'custom',
						value: detail.value === 'custom' ? '' : detail.value || ''
					});
					gui = gui;
				}}
			>
				<svelte:fragment slot="content" let:selectedOption let:selectedRef>
					{#if selectedRef === undefined}
						<span>Select a preset</span>
					{:else if selectedOption?.label}
						<span>{selectedOption.label}</span>
					{:else}
						<span>{selectedRef}</span>
					{/if}
				</svelte:fragment>

				<svelte:fragment slot="option" let:option let:ref>
					<div class="w-full overflow-hidden overflow-ellipsis">
						<div class="text-md break-word">{option.label ? option.label : ref}</div>
					</div>
				</svelte:fragment>
			</DropdownRadio>

			{#if gui?.isFieldPreset(fieldDefinition.binding) === false}
				<Input
					placeholder="Enter value"
					on:change={({ currentTarget }) => {
						if (currentTarget instanceof HTMLInputElement) {
							gui?.saveFieldValue(fieldDefinition.binding, {
								isPreset: false,
								value: currentTarget.value
							});
						}
					}}
				/>
			{/if}
		</div>
	{/each}
{/if}

{#if allDeposits.length > 0}
	<Label class="my-4 whitespace-nowrap text-2xl underline">Deposits</Label>

	{#each allDeposits as deposit}
		<div class="mb-4 flex flex-col gap-2">
			<Label class="whitespace-nowrap text-xl">{tokenInfos.get(deposit.token.address)?.name}</Label>

			<DropdownRadio
				options={{
					...Object.fromEntries(
						deposit.presets.map((preset) => [
							preset,
							{
								label: preset
							}
						])
					),
					...{ custom: { label: 'Custom value' } }
				}}
				on:change={({ detail }) => {
					gui?.saveDeposit(deposit.token_name, detail.value === 'custom' ? '' : detail.value || '');
					gui = gui;
				}}
			>
				<svelte:fragment slot="content" let:selectedOption let:selectedRef>
					{#if selectedRef === undefined}
						<span>Choose deposit amount</span>
					{:else if selectedOption?.label}
						<span>{selectedOption.label}</span>
					{:else}
						<span>{selectedRef}</span>
					{/if}
				</svelte:fragment>

				<svelte:fragment slot="option" let:option let:ref>
					<div class="w-full overflow-hidden overflow-ellipsis">
						<div class="text-md break-word">{option.label ? option.label : ref}</div>
					</div>
				</svelte:fragment>
			</DropdownRadio>

			{#if gui?.isDepositPreset(deposit.token_name) === false}
				<Input
					placeholder="Enter deposit amount"
					on:change={({ currentTarget }) => {
						if (currentTarget instanceof HTMLInputElement) {
							gui?.saveDeposit(deposit.token_name, currentTarget.value);
						}
					}}
				/>
			{/if}
		</div>
	{/each}
{/if}

{#if selectedDeployment}
	<div class="my-4 flex flex-col gap-4">
		<div class="flex items-center gap-2">
			<Checkbox bind:checked={useCustomVaultIds} label="Choose Vault IDs" />
		</div>

		{#if useCustomVaultIds}
			<div class="flex items-center gap-2">
				<Input type="text" placeholder="Enter vault ID" bind:value={customVaultId} />
			</div>
		{/if}

		<Button on:click={handlePopulateVaultIds}>
			{useCustomVaultIds ? 'Set Custom Vault IDs' : 'Populate Random Vault IDs'}
		</Button>
	</div>

	<Button class="flex w-full" on:click={handleAddOrder}>Add Order</Button>
{/if}
