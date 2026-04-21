import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  schema: './src/data/schema/registry.ts',
  out: './migrations/registry',
  dialect: 'sqlite',
})
