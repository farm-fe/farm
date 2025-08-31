import { defineComponent } from "vue";

export default defineComponent({
  name: "vue-jsx-tailwind",
  setup() {
    return () => {
      return (
        <div className="min-h-screen bg-gray-50 p-8">
          <h1 className="text-4xl font-bold text-center mb-12">
            Tailwind CSS Animation Demos
          </h1>
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
          </div>
        </div>
      );
    };
  },
});
