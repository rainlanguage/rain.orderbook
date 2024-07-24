import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
import type { Vault } from '$lib/typeshare/vaultsList';

export const handleDepositGenericModal = () => {
  new ModalVaultDepositGeneric({ target: document.body, props: { open: true } });
};

export const handleDepositModal = (vault: Vault) => {
  new ModalVaultDeposit({ target: document.body, props: { open: true, vault } });
};

export const handleWithdrawModal = (vault: Vault) => {
  new ModalVaultWithdraw({ target: document.body, props: { open: true, vault } });
};
