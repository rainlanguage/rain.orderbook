import TransactionConfirmationModal from '$lib/components/TransactionConfirmationModal.svelte';
import {
	DisclaimerModal,
	type TransactionConfirmationProps,
	type DisclaimerModalProps,
	type VaultActionModalProps
} from '@rainlanguage/ui-components';

import VaultActionModal from '$lib/components/VaultActionModal.svelte';

export const handleVaultActionModal = (props: VaultActionModalProps) => {
	new VaultActionModal({ target: document.body, props });
};

export const handleTransactionConfirmationModal = (props: TransactionConfirmationProps) => {
	new TransactionConfirmationModal({ target: document.body, props });
};

export const handleDisclaimerModal = (props: DisclaimerModalProps) => {
	new DisclaimerModal({ target: document.body, props });
};
