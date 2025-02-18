import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { DepositAndAddOrderCalldataResult } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { OrderSubgraph, Vault } from '@rainlanguage/orderbook/js_api';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
	network: string;
};

export type DepositOrWithdrawArgs = {
	vault: Vault;
	onDepositOrWithdraw: () => void;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
};

export type OrderRemoveArgs = {
	order: OrderSubgraph;
	onRemove: () => void;
	chainId: number;
	orderbookAddress: Hex;
	subgraphUrl: string;
	network: string;
};
