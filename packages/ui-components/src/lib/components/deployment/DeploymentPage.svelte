<script lang="ts">
	import { type ConfigSource } from '../../typeshare/config';
	import type { Writable } from 'svelte/store';
	import DeploymentSteps from './DeploymentSteps.svelte';
	import type { Config } from 'wagmi';
	import type { AppKit } from '@reown/appkit';
	import type {
		ApprovalCalldataResult,
		DepositAndAddOrderCalldataResult,
		DotrainOrderGui
	} from '@rainlanguage/orderbook/js_api';
	import type { Hex } from 'viem';
	import { page } from '$app/stores';
	export let dotrain: string;
	export let key: string;
	export let name: string;
	export let description: string;
	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
	export let handleDeployModal: (args: {
		approvals: ApprovalCalldataResult;
		deploymentCalldata: DepositAndAddOrderCalldataResult;
		orderbookAddress: Hex;
		chainId: number;
		subgraphUrl: string;
	}) => void;
	export let settings: Writable<ConfigSource>;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;
	const stateFromUrl = $page.url.searchParams.get('state') || '';
</script>

<DeploymentSteps
	{dotrain}
	deployment={key}
	deploymentDetails={{ name, description }}
	{wagmiConfig}
	{wagmiConnected}
	{appKitModal}
	{handleDeployModal}
	{stateFromUrl}
	{settings}
	{handleUpdateGuiState}
/>
