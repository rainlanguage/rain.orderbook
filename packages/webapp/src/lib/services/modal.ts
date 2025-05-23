import DeployModal from '$lib/components/DeployModal.svelte';
import DepositOrWithdrawModal from '$lib/components/DepositOrWithdrawModal.svelte';
import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import {
	DisclaimerModal,
	type DepositOrWithdrawModalProps,
	type TransactionConfirmationProps,
	type DisclaimerModalProps,
	type DeployModalProps
} from '@rainlanguage/ui-components';

export const handleDeployModal = (props: DeployModalProps) => {
	new DeployModal({ target: document.body, props });
};

export const handleDepositOrWithdrawModal = (props: DepositOrWithdrawModalProps) => {
	new DepositOrWithdrawModal({ target: document.body, props });
};

export const handleTransactionConfirmationModal = (props: TransactionConfirmationProps) => {
	new TransactionConfirmationModal({ target: document.body, props });
};

export const handleDisclaimerModal = (props: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props });
};
