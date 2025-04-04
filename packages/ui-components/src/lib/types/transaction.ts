import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { Hex } from 'viem';
import type {
	DepositAndAddOrderCalldataResult,
	SgOrder,
	SgVault
} from '@rainlanguage/orderbook/js_api';
import type { Account } from './account';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
	network: string;
	account: Account;
};

export type DepositOrWithdrawArgs = {
	vault: SgVault;
	onDepositOrWithdraw: () => void;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
	account: Account;
};

export type OrderRemoveArgs = {
	order: SgOrder;
	onRemove: () => void;
	chainId: number;
	orderbookAddress: Hex;
	subgraphUrl: string;
	account: Account;
};
