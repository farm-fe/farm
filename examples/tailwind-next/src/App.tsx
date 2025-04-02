import React from 'react';
import logo from './logo.svg?url';
import { ArrowRight, Heart, Menu, Moon, Sun } from 'lucide-react';

const App = () => {
  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <img width={200} src={logo} alt="" />
      <h1 className="text-4xl font-bold text-center mb-12">Tailwind CSS Animation Demos</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 max-w-7xl mx-auto">
        {/* 1. Pulse Effect */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">1. Pulse Effect</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 bg-blue-500 rounded-full animate-pulse"></div>
          </div>
        </div>

        {/* 2. Bounce Effect */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">2. Bounce Effect</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 bg-green-500 rounded-full animate-bounce"></div>
          </div>
        </div>

        {/* 3. Spin Effect */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">3. Spin Effect</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 bg-purple-500 rounded-full animate-spin"></div>
          </div>
        </div>

        {/* 4. Scale on Hover */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">4. Scale on Hover</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 bg-pink-500 rounded-lg transition-transform duration-300 hover:scale-150"></div>
          </div>
        </div>

        {/* 5. Rotate on Hover */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">5. Rotate on Hover</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 bg-yellow-500 rounded-lg transition-transform duration-500 hover:rotate-180"></div>
          </div>
        </div>

        {/* 6. Fade In/Out */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">6. Fade In/Out</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 bg-red-500 rounded-lg animate-[fade_2s_ease-in-out_infinite]"></div>
          </div>
        </div>

        {/* 7. Shake Effect */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">7. Shake Effect</h2>
          <div className="flex justify-center">
            <button className="hover:animate-[shake_0.5s_ease-in-out_infinite] p-3 bg-indigo-500 rounded-lg text-white">
              <Menu className="w-6 h-6" />
            </button>
          </div>
        </div>

        {/* 8. Heart Beat */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">8. Heart Beat</h2>
          <div className="flex justify-center">
            <Heart className="w-8 h-8 text-red-500 animate-[heartbeat_1s_ease-in-out_infinite]" />
          </div>
        </div>

        {/* 9. Slide In */}
        <div className="bg-white p-6 rounded-xl shadow-lg overflow-hidden">
          <h2 className="font-semibold mb-4">9. Slide In</h2>
          <div className="flex justify-center">
            <div className="group flex items-center gap-2 cursor-pointer">
              <span>Hover me</span>
              <ArrowRight className="w-5 h-5 transition-transform duration-300 group-hover:translate-x-2" />
            </div>
          </div>
        </div>

        {/* 10. Color Transition */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">10. Color Transition</h2>
          <div className="flex justify-center">
            <div className="w-12 h-12 rounded-lg transition-colors duration-500 hover:bg-gradient-to-r hover:from-purple-500 hover:to-pink-500 bg-gray-200"></div>
          </div>
        </div>

        {/* 11. Ping Effect */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">11. Ping Effect</h2>
          <div className="flex justify-center">
            <span className="relative flex h-3 w-3">
              <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
              <span className="relative inline-flex rounded-full h-3 w-3 bg-green-500"></span>
            </span>
          </div>
        </div>

        {/* 12. Theme Toggle */}
        <div className="bg-white p-6 rounded-xl shadow-lg">
          <h2 className="font-semibold mb-4">12. Theme Toggle</h2>
          <div className="flex justify-center">
            <div className="relative w-16 h-8 flex items-center cursor-pointer group">
              <div className="w-16 h-8 bg-gray-200 rounded-full p-1 transition-colors duration-300 group-hover:bg-gray-300">
                <div className="w-6 h-6 bg-white rounded-full shadow-md transform transition-transform duration-300 group-hover:translate-x-8 flex items-center justify-center">
                  <Sun className="w-4 h-4 text-yellow-500 absolute group-hover:opacity-0 transition-opacity" />
                  <Moon className="w-4 h-4 text-blue-500 absolute opacity-0 group-hover:opacity-100 transition-opacity" />
                </div>
              </div>
            </div>
          </div>
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


