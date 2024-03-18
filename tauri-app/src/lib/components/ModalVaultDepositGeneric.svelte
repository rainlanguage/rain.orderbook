<script lang="ts">
  import { Button, Modal, Label } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultDeposit, vaultDepositApproveCalldata, vaultDepositCalldata } from '$lib/services/vault';
  import InputToken from '$lib/components/InputToken.svelte';
  import InputVaultId from '$lib/components/InputVaultId.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { ledgerWalletDerivationIndex, ledgerWalletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { orderbookAddress } from '$lib/stores/settings';
  import { walletconnectModal, walletconnectAccount } from '$lib/stores/walletconnect';
  import { checkAllowance, ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';

  export let open = false;
  let vaultId: bigint = 0n;
  let tokenAddress: string = '';
  let tokenDecimals: number = 0;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"

  function reset() {
    vaultId = 0n;
    tokenAddress = '';
    tokenDecimals = 0;
    amount = 0n;
    isSubmitting = false;
    open = false;
    selectWallet = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultDeposit(vaultId, tokenAddress, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    if (!$orderbookAddress) throw Error("Select an orderbook to deposit");
    try {
      const allowance = await checkAllowance(amount, tokenAddress, $orderbookAddress);
      if (!allowance) {
        const approveCalldata = await vaultDepositApproveCalldata(vaultId, tokenAddress, amount) as Uint8Array;
        const approveTx = await ethersExecute(approveCalldata, tokenAddress);
        toasts.success("Approve Transaction sent successfully!");
        await approveTx.wait(1);
      }

      const depositCalldata = await vaultDepositCalldata(vaultId, tokenAddress, amount) as Uint8Array;
      const depositTx = await ethersExecute(depositCalldata, $orderbookAddress);
      toasts.success("Transaction sent successfully!");
      await depositTx.wait(1);

      reset();
    } catch (e) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (typeof e === "object" && (e as any)?.reason) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        toasts.error(`Transaction failed, reason: ${(e as any).reason}`);
      }
      else if (typeof e === "string") toasts.error(e);
      else toasts.error("Transaction failed!");
    }
    isSubmitting = false;
  }
</script>

<Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
  {#if !selectWallet}
    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Vault ID
      </h5>
      <InputVaultId bind:value={vaultId} />
    </div>

    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Token
      </h5>
      <InputToken bind:address={tokenAddress} bind:decimals={tokenDecimals} />
    </div>

    <div class="mb-6">
      <Label
        for="amount"
        class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
      >
        Amount
      </Label>
      <InputTokenAmount bind:value={amount} bind:decimals={tokenDecimals} />
    </div>
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button on:click={() => selectWallet = true} disabled={!amount || amount === 0n || isSubmitting}>
        Proceed
      </Button>
    </div>
  {:else}
    {#if !selectedLedger && !selectedWalletconnect}
      <Button color="alternative" on:click={() => selectWallet = false}>Back</Button>
      <div class="flex flex-col w-full justify-between space-y-2">
        <Button on:click={() => selectedLedger = true}>Ledger Wallet</Button>
        <Button on:click={() => selectedWalletconnect = true}>WalletConnect</Button>
      </div>
    {:else if selectedLedger}
      <Button color="alternative" on:click={() => selectedLedger = false}>Back</Button>
      <InputLedgerWallet
        bind:derivationIndex={$ledgerWalletDerivationIndex}
        bind:walletAddress={$ledgerWalletAddress.value}
      />
      <ButtonLoading on:click={executeLedger} disabled={isSubmitting || !$ledgerWalletAddress || !$ledgerWalletDerivationIndex} loading={isSubmitting}>
        Deposit
      </ButtonLoading>
    {:else if selectedWalletconnect}
      <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
      <div class="text-lg">Note that only <b>mobile</b> wallets are supported.</div>
      <div class="flex flex-col w-full justify-between space-y-2">
        <Button
          color="blue"
          class="px-2 py-1"
          size="xs"
          pill
          on:click={() => $walletconnectModal?.open()}
        >
        {walletconnectLabel}
        </Button>
        <ButtonLoading on:click={executeWalletconnect} disabled={isSubmitting || !$walletconnectAccount} loading={isSubmitting}>
          Deposit
        </ButtonLoading>
      </div>
    {/if}
  {/if}
</Modal>
