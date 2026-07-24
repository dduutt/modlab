# Task 1 Brief: Scaffolding and Theme Setup

## Global Constraints
- Target Window Size: 1000x700
- CSS: Vanilla CSS only (no Tailwind)
- Aesthetics: Premium, modern typography (Inter), subtle animations, curated palettes.
- Testing: Vitest + React Testing Library (TDD)
- OS: Windows target (Tauri)

## Task Details

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `vitest.config.ts`
- Create: `src/styles/theme.css`
- Create: `src/styles/App.css`
- Modify: `src/main.tsx`
- Create: `tests/App.test.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Produces: Base React application with testing infrastructure and global CSS theme variables.

- [ ] **Step 1: Scaffold Vite Project & Install Dependencies**

Run in non-interactive mode:
```bash
npm create vite@latest . -- --template react-ts -y
npm install
npm install -D vitest jsdom @testing-library/react @testing-library/jest-dom
```

- [ ] **Step 2: Configure Vitest**

Create `vitest.config.ts`:
```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    globals: true
  }
})
```

Create `tests/setup.ts`:
```typescript
import '@testing-library/jest-dom';
```

Update `package.json` scripts:
```bash
npm pkg set scripts.test="vitest run"
```

- [ ] **Step 3: Write the failing test**

Create `tests/App.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import App from '../src/App';

describe('App Component', () => {
  it('renders the main application container', () => {
    render(<App />);
    expect(screen.getByTestId('app-container')).toBeInTheDocument();
  });
});
```

- [ ] **Step 4: Run test to verify it fails**

Run: `npm test`
Expected: FAIL

- [ ] **Step 5: Write minimal implementation**

Create `src/styles/theme.css`:
```css
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap');

:root {
  --bg-color: #f8f9fa;
  --panel-bg: #ffffff;
  --text-primary: #111827;
  --text-secondary: #6b7280;
  --border-color: #e5e7eb;
  --accent-color: #3b82f6;
  --accent-hover: #2563eb;
  --danger-color: #ef4444;
  --font-family: 'Inter', sans-serif;
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  --radius-md: 8px;
  --radius-lg: 12px;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: var(--font-family);
  background-color: var(--bg-color);
  color: var(--text-primary);
  height: 100vh;
  overflow: hidden;
}
```

Create `src/styles/App.css`:
```css
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg-color);
}
```

Modify `src/main.tsx`:
```tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './styles/theme.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
```

Modify `src/App.tsx`:
```tsx
import './styles/App.css';

function App() {
  return (
    <div data-testid="app-container" className="app-layout">
      {/* App shell ready */}
    </div>
  );
}

export default App;
```

- [ ] **Step 6: Run test to verify it passes**

Run: `npm test`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add .
git commit -m "chore: scaffold react project, vitest, and theme"
```
