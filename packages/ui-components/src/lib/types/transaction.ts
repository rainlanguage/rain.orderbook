import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { Hex } from 'viem';
import type { DepositAndAddOrderCalldataResult, SgOrder, SgVault } from '@rainlanguage/orderbook';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl?: string;
	network: string;
};

export type VaultActionArgs = {
	vault: SgVault;
	onSuccess: () => void;
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
	account: Hex;
};

export type OrderRemoveArgs = {
	order: SgOrder;
	onRemove: () => void;
	chainId: number;
	orderbookAddress: Hex;
	subgraphUrl: string;
};
