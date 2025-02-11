import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import type {
	ApprovalCalldataResult,
	DepositAndAddOrderCalldataResult,
	OrderSubgraph,
	Vault
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { Config } from 'wagmi';

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
};

export type OrderRemoveModalProps = {
	order: OrderSubgraph;
	onRemove: () => void;
	open?: boolean;
	wagmiConfig: Config;
	chainId: number;
	orderbookAddress: string;
};

export const handleDeployModal = (args: DeployModalProps) => {
	new DeployModal({ target: document.body, props: { open: true, ...args } });
};

export const handleDepositOrWithdrawModal = (args: DepositOrWithdrawModalProps) => {
	new DepositOrWithdrawModal({ target: document.body, props: { open: true, ...args } });
};

export const handleOrderRemoveModal = (args: OrderRemoveModalProps) => {
	new OrderRemoveModal({ target: document.body, props: { open: true, ...args } });
};
