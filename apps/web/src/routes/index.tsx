import { createFileRoute } from '@tanstack/react-router'
import { lazy, Suspense } from 'react'
import { LandingHero } from '../components/landing/LandingHero'
import { RuntimeDiagram } from '../components/landing/RuntimeDiagram'
import { AnimatedShowcase } from '../components/landing/AnimatedShowcase'
import { FeatureGrid } from '../components/landing/FeatureGrid'
import { HowItWorks } from '../components/landing/HowItWorks'
import { LandingCta } from '../components/landing/LandingCta'
import { LandingFooter } from '../components/landing/LandingFooter'

const LandingNav = lazy(() => import('../components/landing/LandingNav'))

export const Route = createFileRoute('/')({ ssr: false, component: LandingPage })

function LandingPage() {
  return (
    <main className="min-h-screen">
      <Suspense fallback={<div className="h-14" />}>
        <LandingNav />
      </Suspense>
      <LandingHero />
      <section className="px-6 py-16 sm:px-10">
        <div className="mb-8 text-center">
          <h2 className="mb-2 font-display text-2xl font-extrabold sm:text-3xl">
            The runtime that connects humans and agents
          </h2>
          <p className="text-[14px] text-muted-foreground">
            Events flow through the kernel. Agents work in isolated workspaces. You see everything in Studio.
          </p>
        </div>
        <RuntimeDiagram />
      </section>
      <AnimatedShowcase />
      <FeatureGrid />
      <HowItWorks />
      <LandingCta />
      <LandingFooter />
    </main>
  )
}
