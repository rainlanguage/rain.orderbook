import DeployModal from '$lib/components/DeployModal.svelte';
import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import DepositModal from '$lib/components/DepositModal.svelte';
import WithdrawModal from '$lib/components/WithdrawModal.svelte';
import {
	DisclaimerModal,
	type VaultActionModalProps,
	type DisclaimerModalProps,
	type DeployModalProps,
	type TransactionConfirmationProps
} from '@rainlanguage/ui-components';

export const handleDeployModal = (props: DeployModalProps) => {
	new DeployModal({ target: document.body, props });
};

export const handleDepositModal = (props: VaultActionModalProps) => {
	new DepositModal({ target: document.body, props });
};

export const handleWithdrawModal = (props: VaultActionModalProps) => {
	new WithdrawModal({ target: document.body, props });
};

export const handleTransactionConfirmationModal = (props: TransactionConfirmationProps) => {
	new TransactionConfirmationModal({ target: document.body, props });
};

export const handleDisclaimerModal = (props: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props });
};
