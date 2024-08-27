import { invoke } from '@tauri-apps/api';

export const getAppCommitSha = async (): Promise<string> => invoke('get_app_commit_sha', {});
