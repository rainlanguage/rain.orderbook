<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import { generateMulticallCalldata, type RaindexVault } from '@rainlanguage/orderbook';
  import { multiVaultsWithdraw } from '$lib/services/vault';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { formatUnits, hexToBytes, type Hex } from 'viem';

  export let open = false;
  export let vaults: RaindexVault[];
  export let onWithdraw: () => void;
  export let onCancel: () => void = () => {};

  let isSubmitting = false;
  let selectWallet = false;

  function reset() {
    open = false;
    if (!isSubmitting) {
      selectWallet = false;
    }
  }

  function close() {
    onCancel();
    reset();
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await multiVaultsWithdraw(vaults);
      onWithdraw();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldatas = await Promise.all(
        vaults.map(async (vault) => {
          const calldata = await vault.getWithdrawCalldata(vault.balance.toString());
          if (calldata.error) {
            throw new Error(calldata.error.readableMsg);
          }
          return calldata.value;
        }),
      );
      const calldata = await generateMulticallCalldata(calldatas);
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      if (!calldata.value || calldata.value.length === 0) {
        throw new Error('No calldata generated');
      }
      const calldataBytes = hexToBytes(
        (calldata.value.startsWith('0x') ? calldata.value : `0x${calldata.value}`) as Hex,
      );
      const orderbook = vaults[0].orderbook; // Assuming all vaults belong to the same orderbook
      const tx = await ethersExecute(calldataBytes, orderbook);
      toasts.success('Transaction sent successfully!');
      await tx.wait(1);
      onWithdraw();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }
</script>

{#if !selectWallet}
  <Modal
    title="Withdraw Multiple Vaults"
    bind:open
    outsideclose={!isSubmitting}
    size="sm"
    on:close={close}
  >
    <div class="space-y-3">
      <div class="max-h-48 space-y-2 overflow-y-auto">
        {#each vaults as vault (vault.id)}
          <div class="flex flex-row items-start justify-between rounded-lg bg-gray-50 p-3">
            <span class="mr-2 truncate font-mono text-xs font-medium text-gray-900">{vault.id}</span
            >
            <span class="whitespace-nowrap text-sm font-semibold text-gray-900">
              {formatUnits(vault.balance, Number(vault.token.decimals ?? 18))}
              &nbsp;
              {vault.token.symbol}
            </span>
          </div>
        {/each}
      </div>
    </div>
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset}>Cancel</Button>

      <Button
        on:click={() => {
          selectWallet = true;
          open = false;
        }}
      >
        Proceed
      </Button>
    </div>
  </Modal>
{/if}

<ModalExecute
  chainId={vaults[0].chainId}
  bind:open={selectWallet}
  onBack={() => (open = true)}
  title="Withdraw from Vault"
  execButtonLabel="Withdraw"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
