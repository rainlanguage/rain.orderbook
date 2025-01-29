import DeployModal from '$lib/components/DeployModal.svelte';
import type {
	ApprovalCalldataResult,
	DepositAndAddOrderCalldataResult,
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';

export type DeployModalProps = {
	approvals: ApprovalCalldataResult;
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
};

export const handleDeployModal = (args: DeployModalProps) => {
	new DeployModal({ target: document.body, props: { open: true, ...args } });
};
