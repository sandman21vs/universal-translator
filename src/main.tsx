// src/main.tsx
import React from 'react';
import { createRoot } from 'react-dom/client';
import App from './ui/App';
import './ui/styles.css';
document.documentElement.dataset.platform = /Mac/.test(navigator.userAgent) ? 'mac' : 'other';
createRoot(document.getElementById('root')!).render(<React.StrictMode><App /></React.StrictMode>);
