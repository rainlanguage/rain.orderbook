
// this file is generated — do not edit it


/// <reference types="@sveltejs/kit" />

/**
 * Environment variables [loaded by Vite](https://vitejs.dev/guide/env-and-mode.html#env-files) from `.env` files and `process.env`. Like [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), this module cannot be imported into client-side code. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * _Unlike_ [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), the values exported from this module are statically injected into your bundle at build time, enabling optimisations like dead code elimination.
 * 
 * ```ts
 * import { API_KEY } from '$env/static/private';
 * ```
 * 
 * Note that all environment variables referenced in your code should be declared (for example in an `.env` file), even if they don't have a value until the app is deployed:
 * 
 * ```
 * MY_FEATURE_FLAG=""
 * ```
 * 
 * You can override `.env` values from the command line like so:
 * 
 * ```bash
 * MY_FEATURE_FLAG="enabled" npm run dev
 * ```
 */
declare module '$env/static/private' {
	export const SHELL: string;
	export const npm_command: string;
	export const LSCOLORS: string;
	export const COREPACK_ENABLE_AUTO_PIN: string;
	export const NIX_CC_WRAPPER_TARGET_TARGET_arm64_apple_darwin: string;
	export const PKG_CONFIG_FOR_TARGET: string;
	export const OBJDUMP_FOR_TARGET: string;
	export const npm_config_userconfig: string;
	export const NIX_CC_WRAPPER_TARGET_BUILD_arm64_apple_darwin: string;
	export const COLORTERM: string;
	export const npm_config_cache: string;
	export const LESS: string;
	export const XPC_FLAGS: string;
	export const NIX_BUILD_CORES: string;
	export const NVM_INC: string;
	export const NIX_GCROOT: string;
	export const NIX_BINTOOLS_WRAPPER_TARGET_HOST_arm64_apple_darwin: string;
	export const TERM_PROGRAM_VERSION: string;
	export const configureFlags: string;
	export const mesonFlags: string;
	export const PKG_CONFIG_PATH: string;
	export const PYTHONNOUSERSITE: string;
	export const shell: string;
	export const SIZE_FOR_TARGET: string;
	export const __sandboxProfile: string;
	export const depsHostHost: string;
	export const NODE: string;
	export const PYTHONHASHSEED: string;
	export const __CFBundleIdentifier: string;
	export const AS_FOR_TARGET: string;
	export const SSH_AUTH_SOCK: string;
	export const CC_FOR_TARGET: string;
	export const WARP_USE_SSH_WRAPPER: string;
	export const STRINGS: string;
	export const LD_FOR_BUILD: string;
	export const LD_FOR_TARGET: string;
	export const depsTargetTarget: string;
	export const OSLogRateLimit: string;
	export const stdenv: string;
	export const COLOR: string;
	export const npm_config_local_prefix: string;
	export const PKG_CONFIG_PATH_FOR_TARGET: string;
	export const NIX_CFLAGS_COMPILE_FOR_BUILD: string;
	export const builder: string;
	export const shellHook: string;
	export const npm_config_globalconfig: string;
	export const NIX_BINTOOLS_FOR_TARGET: string;
	export const NIX_LDFLAGS_FOR_TARGET: string;
	export const CONDA_CHANGEPS1: string;
	export const EDITOR: string;
	export const phases: string;
	export const MACOSX_DEPLOYMENT_TARGET: string;
	export const NIX_PKG_CONFIG_WRAPPER_TARGET_TARGET_arm64_apple_darwin: string;
	export const PWD: string;
	export const SDKROOT: string;
	export const SOURCE_DATE_EPOCH: string;
	export const LOGNAME: string;
	export const NIX_ENFORCE_NO_NATIVE: string;
	export const __propagatedSandboxProfile: string;
	export const npm_config_init_module: string;
	export const STRIP_FOR_TARGET: string;
	export const LaunchInstanceID: string;
	export const NIX_CC_WRAPPER_TARGET_HOST_arm64_apple_darwin: string;
	export const RANLIB_FOR_TARGET: string;
	export const AS_FOR_BUILD: string;
	export const CXX: string;
	export const NIX_APPLE_SDK_VERSION: string;
	export const SYSTEM_CERTIFICATE_PATH: string;
	export const _: string;
	export const TEMPDIR: string;
	export const system: string;
	export const NoDefaultCurrentDirectoryInExePath: string;
	export const STRINGS_FOR_TARGET: string;
	export const SIZE_FOR_BUILD: string;
	export const HOST_PATH: string;
	export const CLAUDECODE: string;
	export const COMMAND_MODE: string;
	export const IN_NIX_SHELL: string;
	export const doInstallCheck: string;
	export const HOME: string;
	export const NIX_BINTOOLS: string;
	export const GETTEXTDATADIRS: string;
	export const AUTOJUMP_ERROR_PATH: string;
	export const LANG: string;
	export const LS_COLORS: string;
	export const NIX_DONT_SET_RPATH: string;
	export const depsTargetTargetPropagated: string;
	export const npm_package_version: string;
	export const SECURITYSESSIONID: string;
	export const cmakeFlags: string;
	export const CXX_FOR_BUILD: string;
	export const NIX_PKG_CONFIG_WRAPPER_TARGET_HOST_arm64_apple_darwin: string;
	export const NIX_SSL_CERT_FILE: string;
	export const LD_DYLD_PATH: string;
	export const outputs: string;
	export const WARP_HONOR_PS1: string;
	export const NIX_STORE: string;
	export const TMPDIR: string;
	export const NIX_CFLAGS_COMPILE_FOR_TARGET: string;
	export const SSH_SOCKET_DIR: string;
	export const LD: string;
	export const NM_FOR_BUILD: string;
	export const NIX_BINTOOLS_FOR_BUILD: string;
	export const buildPhase: string;
	export const AR_FOR_TARGET: string;
	export const COMMIT_SHA: string;
	export const INIT_CWD: string;
	export const STRIP_FOR_BUILD: string;
	export const npm_lifecycle_script: string;
	export const NVM_DIR: string;
	export const doCheck: string;
	export const NIX_DONT_SET_RPATH_FOR_BUILD: string;
	export const npm_config_npm_version: string;
	export const __propagatedImpureHostDeps: string;
	export const depsBuildBuild: string;
	export const PYTHONPATH: string;
	export const TERM: string;
	export const npm_package_name: string;
	export const NIX_NO_SELF_RPATH: string;
	export const ZSH: string;
	export const PATH_LOCALE: string;
	export const SIZE: string;
	export const OBJCOPY_FOR_BUILD: string;
	export const propagatedNativeBuildInputs: string;
	export const npm_config_prefix: string;
	export const CC_FOR_BUILD: string;
	export const USER: string;
	export const strictDeps: string;
	export const AR: string;
	export const AS: string;
	export const TEMP: string;
	export const AUTOJUMP_SOURCED: string;
	export const OBJDUMP_FOR_BUILD: string;
	export const npm_lifecycle_event: string;
	export const SHLVL: string;
	export const NVM_CD_FLAGS: string;
	export const AR_FOR_BUILD: string;
	export const NIX_BUILD_TOP: string;
	export const CXX_FOR_TARGET: string;
	export const NIX_BINTOOLS_WRAPPER_TARGET_TARGET_arm64_apple_darwin: string;
	export const NM: string;
	export const NIX_LDFLAGS_FOR_BUILD: string;
	export const GIT_EDITOR: string;
	export const PAGER: string;
	export const NIX_CFLAGS_COMPILE: string;
	export const __impureHostDeps: string;
	export const patches: string;
	export const ZERO_AR_DATE: string;
	export const NIX_IGNORE_LD_THROUGH_GCC: string;
	export const buildInputs: string;
	export const preferLocalBuild: string;
	export const XPC_SERVICE_NAME: string;
	export const npm_config_user_agent: string;
	export const OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: string;
	export const npm_execpath: string;
	export const LC_CTYPE: string;
	export const NM_FOR_TARGET: string;
	export const OBJCOPY_FOR_TARGET: string;
	export const NODE_PATH: string;
	export const CLAUDE_CODE_ENTRYPOINT: string;
	export const depsBuildTarget: string;
	export const OBJCOPY: string;
	export const DETERMINISTIC_BUILD: string;
	export const RANLIB_FOR_BUILD: string;
	export const WARP_IS_LOCAL_SHELL_SESSION: string;
	export const out: string;
	export const npm_package_json: string;
	export const BUN_INSTALL: string;
	export const STRIP: string;
	export const XDG_DATA_DIRS: string;
	export const TMP: string;
	export const OBJDUMP: string;
	export const npm_config_noproxy: string;
	export const PATH: string;
	export const propagatedBuildInputs: string;
	export const npm_config_node_gyp: string;
	export const dontAddDisableDepTrack: string;
	export const CC: string;
	export const NIX_CC_FOR_TARGET: string;
	export const NIX_CC: string;
	export const depsBuildTargetPropagated: string;
	export const STRINGS_FOR_BUILD: string;
	export const depsBuildBuildPropagated: string;
	export const npm_config_global_prefix: string;
	export const NIX_BINTOOLS_WRAPPER_TARGET_BUILD_arm64_apple_darwin: string;
	export const NVM_BIN: string;
	export const DEVELOPER_DIR: string;
	export const CONFIG_SHELL: string;
	export const __structuredAttrs: string;
	export const npm_node_execpath: string;
	export const RANLIB: string;
	export const NIX_HARDENING_ENABLE: string;
	export const __darwinAllowLocalNetworking: string;
	export const NIX_LDFLAGS: string;
	export const nativeBuildInputs: string;
	export const __CF_USER_TEXT_ENCODING: string;
	export const name: string;
	export const npm_package_engines_node: string;
	export const TERM_PROGRAM: string;
	export const NIX_CC_FOR_BUILD: string;
	export const PKG_CONFIG: string;
	export const depsHostHostPropagated: string;
}

/**
 * Similar to [`$env/static/private`](https://svelte.dev/docs/kit/$env-static-private), except that it only includes environment variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Values are replaced statically at build time.
 * 
 * ```ts
 * import { PUBLIC_BASE_URL } from '$env/static/public';
 * ```
 */
declare module '$env/static/public' {
	
}

/**
 * This module provides access to runtime environment variables, as defined by the platform you're running on. For example if you're using [`adapter-node`](https://github.com/sveltejs/kit/tree/main/packages/adapter-node) (or running [`vite preview`](https://svelte.dev/docs/kit/cli)), this is equivalent to `process.env`. This module only includes variables that _do not_ begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) _and do_ start with [`config.kit.env.privatePrefix`](https://svelte.dev/docs/kit/configuration#env) (if configured).
 * 
 * This module cannot be imported into client-side code.
 * 
 * Dynamic environment variables cannot be used during prerendering.
 * 
 * ```ts
 * import { env } from '$env/dynamic/private';
 * console.log(env.DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 * 
 * > In `dev`, `$env/dynamic` always includes environment variables from `.env`. In `prod`, this behavior will depend on your adapter.
 */
declare module '$env/dynamic/private' {
	export const env: {
		SHELL: string;
		npm_command: string;
		LSCOLORS: string;
		COREPACK_ENABLE_AUTO_PIN: string;
		NIX_CC_WRAPPER_TARGET_TARGET_arm64_apple_darwin: string;
		PKG_CONFIG_FOR_TARGET: string;
		OBJDUMP_FOR_TARGET: string;
		npm_config_userconfig: string;
		NIX_CC_WRAPPER_TARGET_BUILD_arm64_apple_darwin: string;
		COLORTERM: string;
		npm_config_cache: string;
		LESS: string;
		XPC_FLAGS: string;
		NIX_BUILD_CORES: string;
		NVM_INC: string;
		NIX_GCROOT: string;
		NIX_BINTOOLS_WRAPPER_TARGET_HOST_arm64_apple_darwin: string;
		TERM_PROGRAM_VERSION: string;
		configureFlags: string;
		mesonFlags: string;
		PKG_CONFIG_PATH: string;
		PYTHONNOUSERSITE: string;
		shell: string;
		SIZE_FOR_TARGET: string;
		__sandboxProfile: string;
		depsHostHost: string;
		NODE: string;
		PYTHONHASHSEED: string;
		__CFBundleIdentifier: string;
		AS_FOR_TARGET: string;
		SSH_AUTH_SOCK: string;
		CC_FOR_TARGET: string;
		WARP_USE_SSH_WRAPPER: string;
		STRINGS: string;
		LD_FOR_BUILD: string;
		LD_FOR_TARGET: string;
		depsTargetTarget: string;
		OSLogRateLimit: string;
		stdenv: string;
		COLOR: string;
		npm_config_local_prefix: string;
		PKG_CONFIG_PATH_FOR_TARGET: string;
		NIX_CFLAGS_COMPILE_FOR_BUILD: string;
		builder: string;
		shellHook: string;
		npm_config_globalconfig: string;
		NIX_BINTOOLS_FOR_TARGET: string;
		NIX_LDFLAGS_FOR_TARGET: string;
		CONDA_CHANGEPS1: string;
		EDITOR: string;
		phases: string;
		MACOSX_DEPLOYMENT_TARGET: string;
		NIX_PKG_CONFIG_WRAPPER_TARGET_TARGET_arm64_apple_darwin: string;
		PWD: string;
		SDKROOT: string;
		SOURCE_DATE_EPOCH: string;
		LOGNAME: string;
		NIX_ENFORCE_NO_NATIVE: string;
		__propagatedSandboxProfile: string;
		npm_config_init_module: string;
		STRIP_FOR_TARGET: string;
		LaunchInstanceID: string;
		NIX_CC_WRAPPER_TARGET_HOST_arm64_apple_darwin: string;
		RANLIB_FOR_TARGET: string;
		AS_FOR_BUILD: string;
		CXX: string;
		NIX_APPLE_SDK_VERSION: string;
		SYSTEM_CERTIFICATE_PATH: string;
		_: string;
		TEMPDIR: string;
		system: string;
		NoDefaultCurrentDirectoryInExePath: string;
		STRINGS_FOR_TARGET: string;
		SIZE_FOR_BUILD: string;
		HOST_PATH: string;
		CLAUDECODE: string;
		COMMAND_MODE: string;
		IN_NIX_SHELL: string;
		doInstallCheck: string;
		HOME: string;
		NIX_BINTOOLS: string;
		GETTEXTDATADIRS: string;
		AUTOJUMP_ERROR_PATH: string;
		LANG: string;
		LS_COLORS: string;
		NIX_DONT_SET_RPATH: string;
		depsTargetTargetPropagated: string;
		npm_package_version: string;
		SECURITYSESSIONID: string;
		cmakeFlags: string;
		CXX_FOR_BUILD: string;
		NIX_PKG_CONFIG_WRAPPER_TARGET_HOST_arm64_apple_darwin: string;
		NIX_SSL_CERT_FILE: string;
		LD_DYLD_PATH: string;
		outputs: string;
		WARP_HONOR_PS1: string;
		NIX_STORE: string;
		TMPDIR: string;
		NIX_CFLAGS_COMPILE_FOR_TARGET: string;
		SSH_SOCKET_DIR: string;
		LD: string;
		NM_FOR_BUILD: string;
		NIX_BINTOOLS_FOR_BUILD: string;
		buildPhase: string;
		AR_FOR_TARGET: string;
		COMMIT_SHA: string;
		INIT_CWD: string;
		STRIP_FOR_BUILD: string;
		npm_lifecycle_script: string;
		NVM_DIR: string;
		doCheck: string;
		NIX_DONT_SET_RPATH_FOR_BUILD: string;
		npm_config_npm_version: string;
		__propagatedImpureHostDeps: string;
		depsBuildBuild: string;
		PYTHONPATH: string;
		TERM: string;
		npm_package_name: string;
		NIX_NO_SELF_RPATH: string;
		ZSH: string;
		PATH_LOCALE: string;
		SIZE: string;
		OBJCOPY_FOR_BUILD: string;
		propagatedNativeBuildInputs: string;
		npm_config_prefix: string;
		CC_FOR_BUILD: string;
		USER: string;
		strictDeps: string;
		AR: string;
		AS: string;
		TEMP: string;
		AUTOJUMP_SOURCED: string;
		OBJDUMP_FOR_BUILD: string;
		npm_lifecycle_event: string;
		SHLVL: string;
		NVM_CD_FLAGS: string;
		AR_FOR_BUILD: string;
		NIX_BUILD_TOP: string;
		CXX_FOR_TARGET: string;
		NIX_BINTOOLS_WRAPPER_TARGET_TARGET_arm64_apple_darwin: string;
		NM: string;
		NIX_LDFLAGS_FOR_BUILD: string;
		GIT_EDITOR: string;
		PAGER: string;
		NIX_CFLAGS_COMPILE: string;
		__impureHostDeps: string;
		patches: string;
		ZERO_AR_DATE: string;
		NIX_IGNORE_LD_THROUGH_GCC: string;
		buildInputs: string;
		preferLocalBuild: string;
		XPC_SERVICE_NAME: string;
		npm_config_user_agent: string;
		OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE: string;
		npm_execpath: string;
		LC_CTYPE: string;
		NM_FOR_TARGET: string;
		OBJCOPY_FOR_TARGET: string;
		NODE_PATH: string;
		CLAUDE_CODE_ENTRYPOINT: string;
		depsBuildTarget: string;
		OBJCOPY: string;
		DETERMINISTIC_BUILD: string;
		RANLIB_FOR_BUILD: string;
		WARP_IS_LOCAL_SHELL_SESSION: string;
		out: string;
		npm_package_json: string;
		BUN_INSTALL: string;
		STRIP: string;
		XDG_DATA_DIRS: string;
		TMP: string;
		OBJDUMP: string;
		npm_config_noproxy: string;
		PATH: string;
		propagatedBuildInputs: string;
		npm_config_node_gyp: string;
		dontAddDisableDepTrack: string;
		CC: string;
		NIX_CC_FOR_TARGET: string;
		NIX_CC: string;
		depsBuildTargetPropagated: string;
		STRINGS_FOR_BUILD: string;
		depsBuildBuildPropagated: string;
		npm_config_global_prefix: string;
		NIX_BINTOOLS_WRAPPER_TARGET_BUILD_arm64_apple_darwin: string;
		NVM_BIN: string;
		DEVELOPER_DIR: string;
		CONFIG_SHELL: string;
		__structuredAttrs: string;
		npm_node_execpath: string;
		RANLIB: string;
		NIX_HARDENING_ENABLE: string;
		__darwinAllowLocalNetworking: string;
		NIX_LDFLAGS: string;
		nativeBuildInputs: string;
		__CF_USER_TEXT_ENCODING: string;
		name: string;
		npm_package_engines_node: string;
		TERM_PROGRAM: string;
		NIX_CC_FOR_BUILD: string;
		PKG_CONFIG: string;
		depsHostHostPropagated: string;
		[key: `PUBLIC_${string}`]: undefined;
		[key: `${string}`]: string | undefined;
	}
}

/**
 * Similar to [`$env/dynamic/private`](https://svelte.dev/docs/kit/$env-dynamic-private), but only includes variables that begin with [`config.kit.env.publicPrefix`](https://svelte.dev/docs/kit/configuration#env) (which defaults to `PUBLIC_`), and can therefore safely be exposed to client-side code.
 * 
 * Note that public dynamic environment variables must all be sent from the server to the client, causing larger network requests — when possible, use `$env/static/public` instead.
 * 
 * Dynamic environment variables cannot be used during prerendering.
 * 
 * ```ts
 * import { env } from '$env/dynamic/public';
 * console.log(env.PUBLIC_DEPLOYMENT_SPECIFIC_VARIABLE);
 * ```
 */
declare module '$env/dynamic/public' {
	export const env: {
		[key: `PUBLIC_${string}`]: string | undefined;
	}
}
