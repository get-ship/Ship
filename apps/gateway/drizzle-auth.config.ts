import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  schema: './src/data/schema/auth.ts',
  out: './migrations/auth',
  dialect: 'sqlite',
})
