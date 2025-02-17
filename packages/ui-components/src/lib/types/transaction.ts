import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { DepositAndAddOrderCalldataResult } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { Config } from 'wagmi';
import type { OrderSubgraph, Vault } from '@rainlanguage/orderbook/js_api';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
	network: string;
};

export type DepositOrWithdrawModalArgs = {
	vault: Vault;
	onDepositOrWithdraw: () => void;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
};

export type OrderRemoveModalArgs = {
	order: OrderSubgraph;
	onRemove: () => void;
	wagmiConfig: Config;
	chainId: number;
	orderbookAddress: string;
};

export type QuoteDebugModalHandler = (
	order: OrderSubgraph,
	rpcUrl: string,
	orderbook: string,
	inputIOIndex: number,
	outputIOIndex: number,
	pair: string,
	blockNumber?: number
) => void;

export type DebugTradeModalHandler = (hash: string, rpcUrl: string) => void;

export type DisclaimerModalArgs = {
	onAccept: () => void;
};

