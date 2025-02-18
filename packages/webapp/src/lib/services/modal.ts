import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import { DisclaimerModal, type DeploymentArgs } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';

export type DisclaimerModalProps = ComponentProps<DisclaimerModal>;
export type DepositOrWithdrawModalProps = ComponentProps<DepositOrWithdrawModal>;
export type OrderRemoveModalProps = ComponentProps<OrderRemoveModal>;

export const handleDeployModal = (args: Omit<DeploymentArgs, 'open'>) => {
	new DeployModal({ target: document.body, props: { ...args, open: true } });
};

export const handleDepositOrWithdrawModal = (args: Omit<DepositOrWithdrawModalProps, 'open'>) => {
	new DepositOrWithdrawModal({ target: document.body, props: { ...args, open: true } });
};

export const handleOrderRemoveModal = (args: Omit<OrderRemoveModalProps, 'open'>) => {
	new OrderRemoveModal({ target: document.body, props: { ...args, open: true } });
};

export const handleDisclaimerModal = (args: Omit<DisclaimerModalProps, 'open'>) => {
	new DisclaimerModal({ target: document.body, props: { ...args, open: true } });
};
