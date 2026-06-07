import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import Display from './Display';
import './styles/globals.css';

const params = new URLSearchParams(window.location.search);
const isDisplayWindow = params.get('window') === 'display';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    {isDisplayWindow ? <Display /> : <App />}
  </React.StrictMode>,
);
