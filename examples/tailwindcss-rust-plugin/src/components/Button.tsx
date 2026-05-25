import React from 'react';

type Variant = 'primary' | 'secondary' | 'danger';

const variantClass: Record<Variant, string> = {
  primary: 'bg-brand hover:bg-brand-dark text-white',
  secondary: 'bg-gray-200 hover:bg-gray-300 text-gray-900',
  danger: 'bg-red-500 hover:bg-red-600 text-white'
};

interface ButtonProps {
  variant?: Variant;
  label: string;
  testId?: string;
}

const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  label,
  testId
}) => {
  return (
    <button
      data-testid={testId}
      className={`px-4 py-2 rounded font-medium transition-colors duration-150 ${variantClass[variant]}`}
    >
      {label}
    </button>
  );
};

export default Button;
