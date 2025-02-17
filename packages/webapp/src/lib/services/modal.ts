import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import {
	DisclaimerModal,
	type DeploymentArgs,
	type DepositOrWithdrawModalArgs,
	type OrderRemoveModalArgs,
	type DisclaimerModalArgs
} from '@rainlanguage/ui-components';

export const handleDeployModal = (args: Omit<DeploymentArgs, 'open'>) => {
	new DeployModal({ target: document.body, props: { ...args, open: true } });
};

export const handleDepositOrWithdrawModal = (args: Omit<DepositOrWithdrawModalArgs, 'open'>) => {
	new DepositOrWithdrawModal({ target: document.body, props: { ...args, open: true } });
};

export const handleOrderRemoveModal = (args: Omit<OrderRemoveModalArgs, 'open'>) => {
	new OrderRemoveModal({ target: document.body, props: { ...args, open: true } });
};

export const handleDisclaimerModal = (args: Omit<DisclaimerModalArgs, 'open'>) => {
	new DisclaimerModal({ target: document.body, props: { ...args, open: true } });
};
