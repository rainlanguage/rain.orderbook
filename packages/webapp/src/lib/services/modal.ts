import DeployModal from '$lib/components/DeployModal.svelte';
import OrderRemoveModal from '$lib/components/OrderRemoveModal.svelte';
import DepositModal from '$lib/components/DepositModal.svelte';
import WithdrawModal from '$lib/components/WithdrawModal.svelte';
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

export const handleDepositModal = (props: Omit<DepositOrWithdrawModalProps, 'args'> & { args: Omit<DepositOrWithdrawModalProps['args'], 'action'> }) => {
	new DepositModal({ target: document.body, props });
};

export const handleWithdrawModal = (props: Omit<DepositOrWithdrawModalProps, 'args'> & { args: Omit<DepositOrWithdrawModalProps['args'], 'action'> }) => {
	new WithdrawModal({ target: document.body, props });
};

export const handleOrderRemoveModal = (props: OrderRemoveModalProps) => {
	new OrderRemoveModal({ target: document.body, props });
};

export const handleDisclaimerModal = (props: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props });
};
