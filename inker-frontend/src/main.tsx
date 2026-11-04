import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'
import './index.css'
import { client } from './lib/api/client.gen';
import { createClientConfig } from './lib/api-config';

client.setConfig(createClientConfig({}));

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
