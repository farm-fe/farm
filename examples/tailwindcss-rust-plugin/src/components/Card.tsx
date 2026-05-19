import React from 'react';

interface CardProps {
  title: string;
  description: string;
  badge?: string;
  testId?: string;
}

/**
 * Card exercises responsive utilities (md:*), arbitrary values ([&_p]:*),
 * and a custom shadow utility.
 */
const Card: React.FC<CardProps> = ({ title, description, badge, testId }) => {
  return (
    <div
      data-testid={testId}
      className="w-full md:w-1/2 lg:w-1/3 bg-white rounded-lg shadow-md p-6 flex flex-col gap-3 hover:shadow-lg transition-shadow"
    >
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-semibold text-gray-900">{title}</h3>
        {badge ? (
          <span className="text-xs uppercase tracking-wider bg-indigo-100 text-indigo-800 px-2 py-1 rounded">
            {badge}
          </span>
        ) : null}
      </div>
      <p className="text-gray-600 leading-relaxed">{description}</p>
    </div>
  );
};

export default Card;
