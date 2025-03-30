import React, { useState } from 'react'

export const WiggleButton = () => (
  <button className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded animate-wiggle">
    Wiggle Me!
  </button>
)

export const FloatingCard = () => (
  <div className="bg-white p-6 rounded-lg shadow-lg animate-float">
    <h2 className="text-xl font-bold mb-2">Floating Card</h2>
    <p>This card gently floats up and down.</p>
  </div>
)

export const FadeInText = () => {
  const [show, setShow] = useState(false)
  return (
    <div>
      <button
        onClick={() => setShow(!show)}
        className="bg-purple-500 text-white px-4 py-2 rounded mb-4"
      >
        Toggle Fade
      </button>
      {show && (
        <p className="text-2xl font-bold text-purple-600 animate-fadeIn">
          I fade in smoothly!
        </p>
      )}
    </div>
  )
}

export const SlideInPanel = () => {
  const [show, setShow] = useState(false)
  return (
    <div>
      <button
        onClick={() => setShow(!show)}
        className="bg-green-500 text-white px-4 py-2 rounded mb-4"
      >
        Toggle Slide
      </button>
      {show && (
        <div className="bg-green-500 text-white p-4 rounded-lg animate-slideIn">
          <h3 className="text-xl font-bold">Sliding Panel</h3>
          <p>I slide in from the left!</p>
        </div>
      )}
    </div>
  )
}

export const PulsingIcon = () => (
  <div className="w-12 h-12 rounded-full bg-red-500 animate-pulse"></div>
)

export const BouncingBall = () => (
  <div className="w-12 h-12 rounded-full bg-yellow-500 animate-bounce"></div>
)

export const SpinningLoader = () => (
  <div className="w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
)

export const PingingCircle = () => (
  <span className="relative flex h-3 w-3">
    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-sky-400 opacity-75"></span>
    <span className="relative inline-flex rounded-full h-3 w-3 bg-sky-500"></span>
  </span>
)

export const ScalingSquare = () => (
  <div className="w-12 h-12 bg-purple-500 animate-scale"></div>
)

export const ShakingInput = () => (
  <input
    type="text"
    placeholder="Type and watch me shake!"
    className="border-2 border-gray-300 p-2 rounded animate-shake"
  />
)

export const FlippingCard = () => (
  <div className="w-32 h-32 bg-gradient-to-r from-cyan-500 to-blue-500 rounded-lg animate-rotateY"></div>
)

export const TypingText = () => (
  <div className="w-64 overflow-hidden border-r-2 border-black">
    <p className="font-mono text-lg whitespace-nowrap overflow-hidden animate-typing">
      Welcome to animations!
    </p>
  </div>
)

