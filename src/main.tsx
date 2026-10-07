import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './style.css'
import './visual.css'
import './motion.css'
import './end.css'
import { applyAppearance, browserPreferenceStorage, readAppearance } from './appearance'
import { initializeLanguage } from './locale.ts'

// Apply stored preferences before React's first paint.
applyAppearance(readAppearance(browserPreferenceStorage()))
initializeLanguage(browserPreferenceStorage(),document.documentElement)

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><App /></React.StrictMode>,
)
