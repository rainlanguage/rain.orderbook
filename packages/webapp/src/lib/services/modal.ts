import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { DisclaimerModal } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';

export type DisclaimerModalProps = ComponentProps<DisclaimerModal>;
export type DepositOrWithdrawModalProps = ComponentProps<DepositOrWithdrawModal>;
export type OrderRemoveModalProps = ComponentProps<OrderRemoveModal>;
export type DeployModalProps = ComponentProps<DeployModal>;

export const handleDeployModal = (args: DeployModalProps) => {
	new DeployModal({ target: document.body, props: { ...args, open: true } });
};

export const handleDepositOrWithdrawModal = (args: DepositOrWithdrawModalProps) => {
	new DepositOrWithdrawModal({ target: document.body, props: { ...args, open: true } });
};

export const handleOrderRemoveModal = (args: OrderRemoveModalProps) => {
	new OrderRemoveModal({ target: document.body, props: { ...args, open: true } });
};

export const handleDisclaimerModal = (args: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props: { ...args, open: true } });
};
