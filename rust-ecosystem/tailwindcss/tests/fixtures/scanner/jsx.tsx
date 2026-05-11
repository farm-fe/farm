import React from "react";

interface ButtonProps {
  label: string;
  variant?: "primary" | "secondary";
}

export function Button({ label, variant = "primary" }: ButtonProps) {
  const base = "inline-flex items-center justify-center font-medium rounded-md";
  const variants = {
    primary: "bg-indigo-600 hover:bg-indigo-700 text-white px-4 py-2",
    secondary: "bg-gray-200 hover:bg-gray-300 text-gray-900 px-4 py-2",
  };

  return (
    <button className={`${base} ${variants[variant]}`}>
      {label}
    </button>
  );
}

export function Card() {
  return (
    <div className="max-w-sm mx-auto bg-white rounded-xl shadow-lg overflow-hidden">
      <div className="p-6">
        <p className="text-gray-500 text-sm leading-relaxed">
          A simple card component.
        </p>
      </div>
    </div>
  );
}
