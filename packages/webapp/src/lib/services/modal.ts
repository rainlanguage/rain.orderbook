import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import type {
	ApprovalCalldataResult,
	DepositAndAddOrderCalldataResult,
	Vault
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';

export type DeployModalProps = {
	approvals: ApprovalCalldataResult;
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
};

export type DepositOrWithdrawModalProps = {
	vault: Vault;
	onDepositOrWithdraw: () => void;
	action: 'deposit' | 'withdraw';
	chainId: number;
	rpcUrl: string;
	subgraphUrl: string;
};

export const handleDeployModal = (args: DeployModalProps) => {
	new DeployModal({ target: document.body, props: { open: true, ...args } });
};

export const handleDepositOrWithdrawModal = (args: DepositOrWithdrawModalProps) => {
	new DepositOrWithdrawModal({ target: document.body, props: { open: true, ...args } });
};
