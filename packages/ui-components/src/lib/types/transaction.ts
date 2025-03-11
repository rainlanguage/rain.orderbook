import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { DepositAndAddOrderCalldataResult } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { SgOrder, SgVault } from '@rainlanguage/orderbook/js_api';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
};

export type DepositOrWithdrawArgs = {
	vault: SgVault;
	onDepositOrWithdraw: () => void;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
};

export type OrderRemoveArgs = {
	order: SgOrder;
	onRemove: () => void;
	chainId: number;
	orderbookAddress: Hex;
	subgraphUrl: string;
};
