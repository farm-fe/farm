import React from 'react';
import Button from './components/Button';
import Card from './components/Card';
import AppliedAlert from './components/AppliedAlert';
import SwatchGrid from './components/SwatchGrid';

const App = () => {
  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <h1 className="text-4xl font-bold text-center mb-8">
        TailwindCSS Rust Plugin
      </h1>

      <section
        data-testid="alert-section"
        className="max-w-3xl mx-auto mb-8"
      >
        <AppliedAlert message="The @apply directive is processed by the Rust plugin." />
      </section>

      <section
        data-testid="button-section"
        className="max-w-3xl mx-auto bg-white rounded-xl shadow-md p-6 mb-8"
      >
        <p className="text-gray-700 mb-4">
          This example uses the Rust-based TailwindCSS plugin for Farm.
        </p>
        <div className="flex flex-wrap gap-2">
          <Button testId="btn-primary" label="Primary" variant="primary" />
          <Button testId="btn-secondary" label="Secondary" variant="secondary" />
          <Button testId="btn-danger" label="Danger" variant="danger" />
        </div>
      </section>

      <section
        data-testid="card-section"
        className="max-w-5xl mx-auto flex flex-col md:flex-row gap-4 mb-8"
      >
        <Card
          testId="card-utilities"
          title="Utilities"
          badge="v4"
          description="Static and functional utilities (spacing, color, radius) generated entirely in Rust."
        />
        <Card
          testId="card-variants"
          title="Variants"
          badge="hover"
          description="Hover, focus, responsive (sm/md/lg), dark, and arbitrary [&_p] variants are all supported."
        />
        <Card
          testId="card-apply"
          title="@apply"
          badge="rust"
          description="Custom components composed via @apply are inlined by the Rust compiler."
        />
      </section>

      <section data-testid="swatch-section" className="max-w-5xl mx-auto">
        <SwatchGrid />
      </section>
    </div>
  );
};

export default App;
