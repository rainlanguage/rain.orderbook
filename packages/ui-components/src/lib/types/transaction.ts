import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { 
	DepositAndAddOrderCalldataResult, 
	DepositCalldataResult,
	WithdrawCalldataResult,
	ApprovalCalldata,
	RemoveOrderCalldata,
	SgOrder, 
	SgVault 
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { Config } from '@wagmi/core';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
};

export type DeploymentTransactionArgs = DeploymentArgs & { config: Config };

export type DepositOrWithdrawArgs = {
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	vault: SgVault;
	subgraphUrl: string;
};

export type DepositOrWithdrawTransactionArgs = DepositOrWithdrawArgs & { config: Config, approvalCalldata?: ApprovalCalldata, transactionCalldata: DepositCalldataResult | WithdrawCalldataResult };

export type RemoveOrderArgs = {
	order: SgOrder;
	chainId: number;
	orderbookAddress: Hex;
	subgraphUrl: string;
};

export type RemoveOrderTransactionArgs = RemoveOrderArgs & { config: Config, removeOrderCalldata: RemoveOrderCalldata };