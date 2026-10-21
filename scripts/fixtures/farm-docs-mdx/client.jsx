import React from 'react';
import { hydrateRoot } from 'react-dom/client';
import Content from './sample.mdx';
hydrateRoot(document.getElementById('root'), <Content />);
