import React, { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';

const container = document.getElementById('wallet-connect');
const root = createRoot(container!);

root.render(
    <StrictMode>
        <App />
    </StrictMode>
);
