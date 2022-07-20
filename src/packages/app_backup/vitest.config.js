import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		clearMocks: true,
		testTimeout: 25000,
		mockReset: true,
	},
	define: { "process.env.NODE": process.env.NODE }
})