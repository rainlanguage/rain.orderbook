<script lang="ts">
  import { Input, Button, Modal } from 'flowbite-svelte';
  import { SearchOutline, CheckCircleSolid, ChevronDownSolid, ExclamationCircleSolid } from 'flowbite-svelte-icons';
  import type { TokenInfo } from '@rainlanguage/orderbook';
  import { useGui } from '$lib/hooks/useGui';
  import { onMount, tick } from 'svelte';

  export let selectedToken: TokenInfo | null = null;
  export let onSelect: (token: TokenInfo) => void;

  let modalOpen = false;
  let searchQuery = '';
  let tokens: TokenInfo[] = [];
  let isSearching = false;
  let customToken: TokenInfo | null = null;
  let customTokenError = '';
  let isValidatingCustomToken = false;

  const gui = useGui();

  async function loadTokens(search?: string) {
    isSearching = true;
    customToken = null;
    customTokenError = '';

    const result = await gui.getAllTokens(search);
    if (result.error) {
      tokens = [];
    } else {
      tokens = result.value;
    }

    isSearching = false;
  }

  function isValidAddress(address: string): boolean {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  }

  async function validateCustomToken(address: string) {
    if (!isValidAddress(address)) {
      customTokenError = 'Invalid address format';
      return;
    }

    isValidatingCustomToken = true;
    customTokenError = '';
    customToken = null;

    try {
      // Create a temporary token key for validation
      const tempKey = 'temp-validation-token';
      
      // Try to set the token which will validate and fetch info
      await gui.setSelectToken(tempKey, address);
      
      // Get the token info
      const result = await gui.getTokenInfo(tempKey);
      if (result.error) {
        throw new Error(result.error.msg);
      }

      // Create custom token info
      const tokenInfo = result.value;
      customToken = {
        key: tempKey,
        address: tokenInfo.address,
        decimals: tokenInfo.decimals,
        name: tokenInfo.name,
        symbol: tokenInfo.symbol
      };

      // Clean up - unset the temporary token
      await gui.unsetSelectToken(tempKey);
    } catch (error) {
      customTokenError = (error as Error).message || 'Invalid token address';
      customToken = null;
    } finally {
      isValidatingCustomToken = false;
    }
  }

  async function handleSearch(event: Event) {
    const target = event.target as HTMLInputElement;
    searchQuery = target.value;
    
    // Check if the input looks like an address
    if (isValidAddress(searchQuery)) {
      // Load normal tokens first
      await loadTokens();
      // Then validate the custom address
      await validateCustomToken(searchQuery);
    } else {
      // Normal search
      customToken = null;
      customTokenError = '';
      await loadTokens(searchQuery || undefined);
    }
  }

  onMount(() => loadTokens());

  function formatAddress(address: string): string {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  }

  function handleTokenSelect(token: TokenInfo) {
    onSelect(token);
    modalOpen = false;
  }

  $: displayText = selectedToken
    ? `${selectedToken.name} (${selectedToken.symbol})`
    : 'Select a token...';

  $: if (modalOpen) {
    tick().then(() => {
      const input = document.querySelector('.token-search-input') as HTMLInputElement;
      if (input) {
        input.focus();
      }
    });
    // Reset search state when modal opens
    searchQuery = '';
    customToken = null;
    customTokenError = '';
    loadTokens();
  }
</script>

<div class="token-dropdown">
  <div class="relative w-full">
    <Button
      color="alternative"
      class="flex w-full justify-between overflow-hidden overflow-ellipsis pl-4 pr-2 text-left"
      size="lg"
      on:click={() => (modalOpen = true)}
    >
      <div class="flex-grow overflow-hidden">
        <span class="text-gray-900 dark:text-white">{displayText}</span>
      </div>
      <ChevronDownSolid class="ml-2 h-4 w-4 text-black dark:text-white" />
    </Button>

    <Modal bind:open={modalOpen} size="md" class="w-full max-w-lg">
      <div slot="header" class="flex w-full items-center justify-between">
        <h3 class="text-xl font-medium text-gray-900 dark:text-white">Select a token</h3>
      </div>
      <div class="relative w-full border-b border-gray-200 p-2 dark:border-gray-600">
        <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-5">
          <SearchOutline class="h-4 w-4 text-gray-500 dark:text-gray-400" />
        </div>
        <Input
          type="text"
          placeholder="Search tokens or enter address (0x...)"
          bind:value={searchQuery}
          on:input={handleSearch}
          class="token-search-input pl-10"
        />
      </div>

      <div class="token-list max-h-80 overflow-y-auto">
        {#if isSearching || isValidatingCustomToken}
          <div class="p-4 text-center text-gray-500 dark:text-gray-400">
            <p>{isValidatingCustomToken ? 'Validating token...' : 'Searching tokens...'}</p>
          </div>
        {:else}
          <!-- Custom token result (if found) -->
          {#if customToken}
            <div
              class="token-item flex cursor-pointer items-center border-b border-gray-100 p-3 bg-yellow-50 border-l-4 border-l-yellow-500 dark:bg-yellow-900/20 dark:border-gray-600"
              on:click={() => customToken && handleTokenSelect(customToken)}
              on:keydown={(e) => e.key === 'Enter' && customToken && handleTokenSelect(customToken)}
              role="button"
              tabindex="0"
            >
              <div class="token-info flex-grow">
                <div class="token-name font-medium text-gray-900 dark:text-white">
                  {customToken.name}
                </div>
                <div class="token-details flex gap-2 text-sm text-gray-500 dark:text-gray-400">
                  <span class="symbol font-medium">{customToken.symbol}</span>
                  <span class="address">{formatAddress(customToken.address)}</span>
                </div>
                <div class="flex items-center gap-1 mt-1">
                  <ExclamationCircleSolid class="h-3 w-3 text-yellow-600" />
                  <span class="text-xs text-yellow-700 dark:text-yellow-400">Custom token (not in list)</span>
                </div>
              </div>
              {#if selectedToken?.address === customToken.address}
                <CheckCircleSolid class="selected-icon h-5 w-5 text-green-500" />
              {/if}
            </div>
          {/if}

          <!-- Custom token error -->
          {#if customTokenError}
            <div class="p-3 bg-red-50 border-l-4 border-l-red-500 dark:bg-red-900/20">
              <div class="flex items-center gap-2 text-red-700 dark:text-red-400">
                <ExclamationCircleSolid class="h-4 w-4" />
                <span class="text-sm">{customTokenError}</span>
              </div>
            </div>
          {/if}

          <!-- Regular token list -->
          {#each tokens as token (token.address)}
            <div
              class="token-item flex cursor-pointer items-center border-b border-gray-100 p-3 last:border-b-0 hover:bg-gray-50 dark:border-gray-600 dark:hover:bg-gray-700"
              class:bg-blue-50={selectedToken?.address === token.address}
              class:dark:bg-blue-900={selectedToken?.address === token.address}
              class:border-l-4={selectedToken?.address === token.address}
              class:border-l-blue-500={selectedToken?.address === token.address}
              on:click={() => handleTokenSelect(token)}
              on:keydown={(e) => e.key === 'Enter' && handleTokenSelect(token)}
              role="button"
              tabindex="0"
            >
              <div class="token-info flex-grow">
                <div class="token-name font-medium text-gray-900 dark:text-white">
                  {token.name}
                </div>
                <div class="token-details flex gap-2 text-sm text-gray-500 dark:text-gray-400">
                  <span class="symbol font-medium">{token.symbol}</span>
                  <span class="address">{formatAddress(token.address)}</span>
                </div>
              </div>
              {#if selectedToken?.address === token.address}
                <CheckCircleSolid class="selected-icon h-5 w-5 text-green-500" />
              {/if}
            </div>
          {/each}

          {#if tokens.length === 0 && !customToken && !customTokenError && !isValidAddress(searchQuery)}
            <div class="no-results p-4 text-center text-gray-500 dark:text-gray-400">
              <p>No tokens found matching your search.</p>
              <button
                class="mt-2 text-blue-600 underline hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
                on:click={() => {
                  searchQuery = '';
                  loadTokens();
                }}
              >
                Clear search
              </button>
            </div>
          {/if}
        {/if}
      </div>
    </Modal>
  </div>
</div>
