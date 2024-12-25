import React from 'react'
import {
  WiggleButton,
  FloatingCard,
  FadeInText,
  SlideInPanel,
  PulsingIcon,
  BouncingBall,
  SpinningLoader,
  PingingCircle,
  ScalingSquare,
  ShakingInput,
  FlippingCard,
  TypingText
} from './AnimatedComponents'

export default function App() {
  return (
    <div className="min-h-screen bg-gray-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-4xl font-bold text-center mb-12">12 Tailwind CSS Animations</h1>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          <AnimationCard title="3. Fade In Text" component={<FadeInText />} />
          <AnimationCard title="4. Slide In Panel" component={<SlideInPanel />} />
          <AnimationCard title="5. Pulsing Icon" component={<PulsingIcon />} />
          <AnimationCard title="6. Bouncing Ball" component={<BouncingBall />} />
          <AnimationCard title="7. Spinning Loader" component={<SpinningLoader />} />
          <AnimationCard title="8. Pinging Circle" component={<PingingCircle />} />
        </div>
      </div>
    </div>
  )
}

const AnimationCard = ({ title, component }: { title: string, component: React.ReactNode }) => (
  <div className="bg-white p-6 rounded-lg shadow-md">
    <h2 className="text-xl font-semibold mb-4">{title}</h2>
    <div className="flex items-center justify-center h-40">
      {component}
    </div>
  </div>
)


