import React from 'react';

const App = () => {
  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <h1 className="text-4xl font-bold text-center mb-8">
        TailwindCSS Rust Plugin
      </h1>
      <div className="max-w-md mx-auto bg-white rounded-xl shadow-md p-6">
        <p className="text-gray-700 mb-4">
          This example uses the Rust-based TailwindCSS plugin for Farm.
        </p>
        <div className="flex gap-2">
          <span className="bg-blue-500 text-white px-3 py-1 rounded">Blue</span>
          <span className="bg-green-500 text-white px-3 py-1 rounded">Green</span>
          <span className="bg-red-500 text-white px-3 py-1 rounded">Red</span>
        </div>
      </div>
    </div>
  );
};

export default App;
