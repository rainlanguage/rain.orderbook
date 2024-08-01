import { render, screen, waitFor } from '@testing-library/svelte';
import { test, vi } from 'vitest';
import { expect } from '$lib/test/matchers';
import { QueryClient } from '@tanstack/svelte-query';
import LightweightChart from './LightweightChart.svelte';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { Vault } from '$lib/typeshare/vaultDetail';
import { goto } from '$app/navigation';
import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';

test('shows query data in correct places', async () => {});
