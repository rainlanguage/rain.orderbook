import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
import ModalVaultDepositGeneric from '$lib/components/ModalVaultDepositGeneric.svelte';
import type { Vault } from '$lib/typeshare/vaultsList';
import ModalOrderRemove from '$lib/components/modal/ModalOrderRemove.svelte';
import type { Order as OrderDetailOrder } from '$lib/typeshare/orderDetail';
import type { Order as OrderListOrder } from '$lib/typeshare/ordersList';

export const handleDepositGenericModal = () => {
  new ModalVaultDepositGeneric({ target: document.body, props: { open: true } });
};

export const handleDepositModal = (vault: Vault) => {
  new ModalVaultDeposit({ target: document.body, props: { open: true, vault } });
};

export const handleWithdrawModal = (vault: Vault) => {
  new ModalVaultWithdraw({ target: document.body, props: { open: true, vault } });
};

export const handleOrderRemoveModal = (order: OrderDetailOrder | OrderListOrder) => {
  new ModalOrderRemove({ target: document.body, props: { order } });
};
