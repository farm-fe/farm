import React from 'react';

const swatches = [
  { name: 'red', cls: 'bg-red-500' },
  { name: 'orange', cls: 'bg-orange-500' },
  { name: 'amber', cls: 'bg-amber-500' },
  { name: 'green', cls: 'bg-green-500' },
  { name: 'teal', cls: 'bg-teal-500' },
  { name: 'sky', cls: 'bg-sky-500' },
  { name: 'indigo', cls: 'bg-indigo-500' },
  { name: 'purple', cls: 'bg-purple-500' }
];

/**
 * Renders a grid of color swatches. Exercises:
 *  - grid + responsive `md:grid-cols-*`
 *  - functional color utilities driven by the design-system theme
 */
const SwatchGrid: React.FC = () => {
  return (
    <div
      data-testid="swatch-grid"
      className="grid grid-cols-4 md:grid-cols-8 gap-2 mt-6"
    >
      {swatches.map((s) => (
        <div
          key={s.name}
          data-color={s.name}
          className={`${s.cls} h-10 rounded`}
        />
      ))}
    </div>
  );
};

export default SwatchGrid;
