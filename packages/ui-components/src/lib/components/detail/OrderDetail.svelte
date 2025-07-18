<script lang="ts">
	import Hash, { HashType } from '../Hash.svelte';
	import BadgeActive from '../BadgeActive.svelte';
	import OrderTradesChart from '../charts/OrderTradesChart.svelte';
	import OrderTradesListTable from '../tables/OrderTradesListTable.svelte';
	import TanstackOrderQuote from './TanstackOrderQuote.svelte';
	import TanstackPageContentDetail from './TanstackPageContentDetail.svelte';
	import CardProperty from '../CardProperty.svelte';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import ButtonVaultLink from '../ButtonVaultLink.svelte';
	// import OrderVaultsVolTable from '../tables/OrderVaultsVolTable.svelte';
	import { QKEY_ORDER } from '../../queries/keys';
	import CodeMirrorRainlang from '../CodeMirrorRainlang.svelte';
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import { Button, TabItem, Tabs, Tooltip } from 'flowbite-svelte';
	import { onDestroy } from 'svelte';
	// import OrderApy from '../tables/OrderAPY.svelte';
	import type { DebugTradeModalHandler, QuoteDebugModalHandler } from '../../types/modal';
	import Refresh from '../icon/Refresh.svelte';
	import { invalidateTanstackQueries } from '$lib/queries/queryClient';
	import {
		ArrowDownToBracketOutline,
		ArrowUpFromBracketOutline,
		InfoCircleOutline
	} from 'flowbite-svelte-icons';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import {
		RaindexClient,
		RaindexOrder,
		RaindexVault,
		type Address,
		type Hex
	} from '@rainlanguage/orderbook';
	import { useToasts } from '$lib/providers/toasts/useToasts';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	export let handleQuoteDebugModal: QuoteDebugModalHandler | undefined = undefined;
	export let handleDebugTradeModal: DebugTradeModalHandler | undefined = undefined;
	export let colorTheme;
	export let codeMirrorTheme;
	export let lightweightChartsTheme;
	export let orderbookAddress: Address;
	export let orderHash: Hex;
	export let chainId: number;
	export let rpcs: string[] | undefined = undefined;

	/** Callback function when remove action is triggered for an order
	 * @param order The order to remove
	 */
	export let onRemove: (raindexClient: RaindexClient, order: RaindexOrder) => void;

	/** Callback function when deposit action is triggered for a vault
	 * @param vault The vault to deposit into
	 */
	export let onDeposit: (raindexClient: RaindexClient, vault: RaindexVault) => void;

	/** Callback function when withdraw action is triggered for a vault
	 * @param vault The vault to withdraw from
	 */
	export let onWithdraw: (raindexClient: RaindexClient, vault: RaindexVault) => void;

	/** Callback function when withdraw all action is triggered for an order
	 * @param vaults The vaults to withdraw from
	 */
	export let onWithdrawAll: (raindexClient: RaindexClient, vaults: RaindexVault[]) => void;

	let codeMirrorDisabled = true;
	let codeMirrorStyles = {};

	const queryClient = useQueryClient();
	const { matchesAccount } = useAccount();
	const { errToast } = useToasts();
	const raindexClient = useRaindexClient();

	$: orderDetailQuery = createQuery<RaindexOrder>({
		queryKey: [orderHash, QKEY_ORDER + orderHash],
		queryFn: async () => {
			const result = await raindexClient.getOrderByHash(chainId, orderbookAddress, orderHash);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		}
	});

	const interval = setInterval(async () => {
		await invalidateTanstackQueries(queryClient, [orderHash]);
	}, 10000);

	onDestroy(() => {
		clearInterval(interval);
	});

	const handleRefresh = async () => {
		try {
			await invalidateTanstackQueries(queryClient, [orderHash]);
		} catch {
			errToast('Failed to refresh');
		}
	};

	enum VaultType {
		Output = 'output',
		Input = 'input',
		InputOutput = 'inputOutput'
	}
	type VaultsGroupedByType = {
		[VaultType.Output]: RaindexVault[];
		[VaultType.Input]: RaindexVault[];
		[VaultType.InputOutput]: RaindexVault[];
	};
	const vaultTypes = [
		{ key: 'Output vaults', type: VaultType.Output },
		{ key: 'Input vaults', type: VaultType.Input },
		{ key: 'Input & output vaults', type: VaultType.InputOutput }
	];
	const getDefaultVaultsGroupedByType = (): VaultsGroupedByType => ({
		[VaultType.Output]: [],
		[VaultType.Input]: [],
		[VaultType.InputOutput]: []
	});

	$: vaultsGroupedByTypes =
		$orderDetailQuery?.data?.vaults?.reduce((acc, vault) => {
			const type = vault.vaultType || 'inputOutput';
			if (!acc[type]) acc[type] = [];
			acc[type].push(vault);
			return acc;
		}, getDefaultVaultsGroupedByType()) || getDefaultVaultsGroupedByType();

	const getWithdrawAllLabel = (type: VaultType) => {
		switch (type) {
			case VaultType.Output:
				return 'Withdraw outputs';
			case VaultType.Input:
				return 'Withdraw inputs';
			case VaultType.InputOutput:
				return 'Withdraw all';
			default:
				return 'Withdraw all vaults';
		}
	};

	// It returns vaults of single group filtered by owner and non-zero balance
	// It makes it possible to withdraw multiple vaults of the same type at once
	// Even if some vaults of that type has zero balance
	const getVaultsGroupFiltered = (type: VaultType) => {
		return vaultsGroupedByTypes[type].filter(
			(vault) => matchesAccount(vault.owner) && vault.balance > 0n
		);
	};

	// It checks does it make sense to display Withdraw button for the vault type
	const shouldShowWithdrawByType = (type: VaultType) => {
		const filteredVaults = getVaultsGroupFiltered(type);
		return filteredVaults.length > 1;
	};
	// If checks does it make sense to display Withdraw all button below all vaults
	const shouldShowWithdrawAll = () => {
		if (!$orderDetailQuery.data?.vaults) return false;
		const { vaults } = $orderDetailQuery.data;
		const filteredVaults = vaults.filter(
			(vault) => matchesAccount(vault.owner) && vault.balance > 0n
		);
		return (
			filteredVaults.length > 1 &&
			getVaultsGroupFiltered(VaultType.InputOutput).length !== filteredVaults.length
		);
	};
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
	<svelte:fragment slot="top" let:data>
		<div
			class="flex w-full flex-wrap items-center justify-between gap-4 text-3xl font-medium lg:justify-between dark:text-white"
		>
			<div class="flex items-center gap-x-2">
				<div class="flex gap-x-2">
					<span class="font-light">Order</span>
					<Hash shorten value={data.orderHash} />
				</div>
				<BadgeActive active={data.active} large />
			</div>

			<div class="flex items-center gap-2">
				{#if matchesAccount(data.owner)}
					{#if data.active}
						<Button
							on:click={() => onRemove(raindexClient, data)}
							data-testid="remove-button"
							aria-label="Remove order">Remove</Button
						>
					{/if}
				{/if}

				<Refresh
					testId="top-refresh"
					on:click={handleRefresh}
					spin={$orderDetailQuery.isLoading || $orderDetailQuery.isFetching}
				/>
			</div>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<div class="flex flex-col gap-y-6">
			<CardProperty>
				<svelte:fragment slot="key">Orderbook</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Identifier} shorten={false} value={data.orderbook} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Owner</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Wallet} shorten={false} value={data.owner} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Created</svelte:fragment>
				<svelte:fragment slot="value">
					{formatTimestampSecondsAsLocal(data.timestampAdded)}
				</svelte:fragment>
			</CardProperty>

			{#each vaultTypes as { key, type }}
				{@const filteredVaults = vaultsGroupedByTypes[type]}
				{#if filteredVaults.length !== 0}
					<CardProperty>
						<svelte:fragment slot="key">
							<div class="flex items-center justify-between gap-x-2">
								<div>
									{key}
									{#if type === VaultType.InputOutput}
										<InfoCircleOutline class="inline h-4 w-4" />
										<Tooltip class="z-10">
											{'These vaults can be an input or an output for this order'}
										</Tooltip>
									{/if}
								</div>
								<div>
									{#if shouldShowWithdrawByType(type)}
										<Button
											size="xs"
											on:click={() => onWithdrawAll(raindexClient, getVaultsGroupFiltered(type))}
											data-testid={`withdraw-all-${type}`}
										>
											<ArrowUpFromBracketOutline size="xs" class="mr-2" />
											{getWithdrawAllLabel(type)}
										</Button>
									{/if}
								</div>
							</div>
						</svelte:fragment>
						<svelte:fragment slot="value">
							<div class="mt-2 space-y-2">
								{#each filteredVaults as vault}
									<ButtonVaultLink tokenVault={vault} {chainId} {orderbookAddress}>
										<svelte:fragment slot="buttons">
											{#if matchesAccount(vault.owner)}
												<div class="flex gap-1">
													<Button
														color="light"
														size="xs"
														data-testid="deposit-button"
														on:click={() => onDeposit(raindexClient, vault)}
													>
														<ArrowDownToBracketOutline size="xs" />
													</Button>
													<Button
														color="light"
														size="xs"
														data-testid="withdraw-button"
														on:click={() => onWithdraw(raindexClient, vault)}
													>
														<ArrowUpFromBracketOutline size="xs" />
													</Button>
												</div>
											{/if}
										</svelte:fragment>
									</ButtonVaultLink>
								{/each}
							</div>
						</svelte:fragment>
					</CardProperty>
				{/if}
			{/each}
			<!-- If there are different vault types — show additional withdraw all button below all vaults -->
			{#if shouldShowWithdrawAll()}
				<Button
					size="xs"
					on:click={() => onWithdrawAll(raindexClient, data.vaults)}
					data-testid="withdraw-all-button"
				>
					<ArrowUpFromBracketOutline class="mr-2" />
					Withdraw all vaults
				</Button>
			{/if}
		</div>
	</svelte:fragment>
	<svelte:fragment slot="chart" let:data>
		<OrderTradesChart order={data} {lightweightChartsTheme} {colorTheme} />
	</svelte:fragment>
	<svelte:fragment slot="below" let:data>
		<TanstackOrderQuote order={data} {handleQuoteDebugModal} />
		<Tabs
			style="underline"
			contentClass="mt-4"
			defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4 list-none"
		>
			<TabItem title="Rainlang source">
				<div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
					<CodeMirrorRainlang
						order={data}
						codeMirrorTheme={$codeMirrorTheme}
						{codeMirrorDisabled}
						{codeMirrorStyles}
					></CodeMirrorRainlang>
				</div>
			</TabItem>
			<TabItem open title="Trades">
				<OrderTradesListTable order={data} {handleDebugTradeModal} {rpcs} />
			</TabItem>
			<TabItem title="Volume">
				<div>TODO: Issue #1989</div>
				<!-- <OrderVaultsVolTable order={data} /> -->
			</TabItem>
			<TabItem title="APY">
				<div>TODO: Issue #1989</div>
				<!-- <OrderApy order={data} /> -->
			</TabItem>
		</Tabs>
	</svelte:fragment>
</TanstackPageContentDetail>
