<script lang="ts">
	import { DropdownRadio, Checkbox, DeploymentSteps } from '@rainlanguage/ui-components';
	import {
		DotrainOrderGui,
		type ApprovalCalldataResult,
		type AvailableDeployments,
		type DepositAndAddOrderCalldataResult,
		type GuiDeposit,
		type GuiFieldDefinition,
		type SelectTokens,
		type TokenInfos,
		type Vault
	} from '@rainlanguage/orderbook/js_api';
	import { Label } from 'flowbite-svelte';
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

	let isLimitStrat = false;
	$: if (isLimitStrat) {
		loadLimit();
	} else {
		loadDca();
	}

	let dotrain = '';
	async function loadDca() {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.webapp/refs/heads/main/public/_strategies/raindex/2-dynamic-spread/dynamic-spread.rain'
		);
		dotrain = await response.text();
	}
	async function loadLimit() {
		const response = await fetch(
			'https://raw.githubusercontent.com/findolor/sample-dotrains/refs/heads/main/fixed-ratio-limit.rain'
		);
		dotrain = await response.text();
	}

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
			initializeVaultIdArrays();
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

	let selectTokens: SelectTokens = new Map();
	function getSelectTokens() {
		if (!gui) return;
		selectTokens = gui.getSelectTokens();
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

	let allTokenInputs: Vault[] = [];
	function getAllTokenInputs() {
		if (!gui) return;
		allTokenInputs = gui.getCurrentDeployment().deployment.order.inputs;
	}

	let allTokenOutputs: Vault[] = [];
	function getAllTokenOutputs() {
		if (!gui) return;
		allTokenOutputs = gui.getCurrentDeployment().deployment.order.outputs;
	}

	$: if (gui) {
		getTokenInfos();
		if (isLimitStrat) getSelectTokens();
		getAllFieldDefinitions();
		getDeposits();
		getAllTokenInputs();
		getAllTokenOutputs();
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

			// @ts-expect-error window.ethereum is not typed
			await window.ethereum?.request({ method: 'eth_requestAccounts' });
			const walletClient = createWalletClient({
				chain: getChainById(
					gui.getCurrentDeployment().deployment.order.network['chain-id'] as number
				),
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

	let inputVaultIds: string[] = [];
	let outputVaultIds: string[] = [];
	function initializeVaultIdArrays() {
		if (!gui) return;
		const deployment = gui.getCurrentDeployment();
		inputVaultIds = new Array(deployment.deployment.order.inputs.length).fill('');
		outputVaultIds = new Array(deployment.deployment.order.outputs.length).fill('');
	}
</script>

<div class="flex h-screen flex-col gap-4">
	<div class="mb-4 flex items-center gap-2">
		<Checkbox
			bind:checked={isLimitStrat}
			label="Is Limit Strategy"
			on:change={() => {
				gui = undefined;
			}}
		/>
	</div>

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

	<div class="flex-grow">
		{#if gui}
			<DeploymentSteps
				{gui}
				{isLimitStrat}
				{inputVaultIds}
				{outputVaultIds}
				{handleAddOrder}
				{tokenInfos}
				{selectTokens}
				{allFieldDefinitions}
				{allTokenInputs}
				{allTokenOutputs}
				{allDeposits}
			/>
		{/if}
	</div>
</div>
