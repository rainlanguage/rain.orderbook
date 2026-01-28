export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["logo.svg"]),
	mimeTypes: {".svg":"image/svg+xml"},
	_: {
		client: {"start":"_app/immutable/entry/start.DOgY5xjP.js","app":"_app/immutable/entry/app.DE_S9Sb5.js","imports":["_app/immutable/entry/start.DOgY5xjP.js","_app/immutable/chunks/entry.B-EOmEDc.js","_app/immutable/chunks/scheduler.D5AhQk_9.js","_app/immutable/chunks/index.CVS4X9MZ.js","_app/immutable/entry/app.DE_S9Sb5.js","_app/immutable/chunks/preload-helper.CmsKOCeN.js","_app/immutable/chunks/sentry.zpUroXQM.js","_app/immutable/chunks/stores.DVn3eyy9.js","_app/immutable/chunks/entry.B-EOmEDc.js","_app/immutable/chunks/scheduler.D5AhQk_9.js","_app/immutable/chunks/index.CVS4X9MZ.js","_app/immutable/chunks/index.Y9UT4V4X.js"],"stylesheets":[],"fonts":[],"uses_env_dynamic_public":false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js')),
			__memo(() => import('./nodes/4.js')),
			__memo(() => import('./nodes/5.js')),
			__memo(() => import('./nodes/6.js')),
			__memo(() => import('./nodes/7.js')),
			__memo(() => import('./nodes/8.js')),
			__memo(() => import('./nodes/9.js'))
		],
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/license",
				pattern: /^\/license\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/orders",
				pattern: /^\/orders\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/orders/add",
				pattern: /^\/orders\/add\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 6 },
				endpoint: null
			},
			{
				id: "/orders/[chainId]-[orderbook]-[orderHash]",
				pattern: /^\/orders\/([^/]+?)-([^/]+?)-([^/]+?)\/?$/,
				params: [{"name":"chainId","optional":false,"rest":false,"chained":false},{"name":"orderbook","optional":false,"rest":false,"chained":false},{"name":"orderHash","optional":false,"rest":false,"chained":false}],
				page: { layouts: [0,], errors: [1,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/settings",
				pattern: /^\/settings\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 7 },
				endpoint: null
			},
			{
				id: "/vaults",
				pattern: /^\/vaults\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 8 },
				endpoint: null
			},
			{
				id: "/vaults/[chainId]-[orderbook]-[id]",
				pattern: /^\/vaults\/([^/]+?)-([^/]+?)-([^/]+?)\/?$/,
				params: [{"name":"chainId","optional":false,"rest":false,"chained":false},{"name":"orderbook","optional":false,"rest":false,"chained":false},{"name":"id","optional":false,"rest":false,"chained":false}],
				page: { layouts: [0,], errors: [1,], leaf: 9 },
				endpoint: null
			}
		],
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
