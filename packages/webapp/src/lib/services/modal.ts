import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import {
	DisclaimerModal,
	type DepositOrWithdrawModalProps,
	type OrderRemoveModalProps,
	type DisclaimerModalProps,
	type DeployModalProps
} from '@rainlanguage/ui-components';

export const handleDeployModal = (props: DeployModalProps) => {
	new DeployModal({ target: document.body, props });
};

export const handleDepositOrWithdrawModal = (props: DepositOrWithdrawModalProps) => {
	new DepositOrWithdrawModal({ target: document.body, props });
};

export const handleOrderRemoveModal = (props: OrderRemoveModalProps) => {
	new OrderRemoveModal({ target: document.body, props });
};

export const handleDisclaimerModal = (props: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props });
};
