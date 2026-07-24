# Modbus Tauri App Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the frontend design for a premium Modbus multi-slave/master Windows desktop application, referencing the provided mockup while optimizing visual aesthetics. 

**Architecture:** 
A React + TypeScript SPA served via Vite, intended to be wrapped by a Tauri shell. The UI is componentized into TabBar, Toolbar, ConfigRow, DataGrid, and StatusBar. State will be managed at the App level initially. Styling uses modular Vanilla CSS with CSS variables for a premium, consistent look (glassmorphism, subtle shadows, modern fonts).

**Tech Stack:** React, TypeScript, Vite, Vitest, React Testing Library, Vanilla CSS.

## Global Constraints

- Tauri Window Size defaults to 1000x700. (Configure via `src-tauri/tauri.conf.json` during Tauri wrap).
- Use Vanilla CSS strictly (no Tailwind).
- Ensure premium design: curated color palettes, modern typography (Inter), smooth hover transitions.
- Testing: Vitest + React Testing Library (TDD approach).
- OS: Windows target (Tauri desktop app).

---

### Task 1: Scaffolding and Theme Setup

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
Expected: FAIL (or error if App doesn't have the testid yet)

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

### Task 2: Tab Navigation Component

**Files:**
- Create: `src/components/TabBar.tsx`
- Create: `src/styles/TabBar.css`
- Create: `tests/TabBar.test.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: Theme variables.
- Produces: `<TabBar />` component.

- [ ] **Step 1: Write the failing test**

Create `tests/TabBar.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import TabBar from '../src/components/TabBar';

describe('TabBar Component', () => {
  it('renders a tab and an add button', () => {
    render(<TabBar tabs={['Slave 2']} activeTab="Slave 2" onTabChange={vi.fn()} onAddTab={vi.fn()} />);
    expect(screen.getByText('Slave 2')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '+' })).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Create `src/styles/TabBar.css`:
```css
.tab-bar {
  display: flex;
  align-items: center;
  background: var(--panel-bg);
  padding: 8px 16px 0;
  border-bottom: 1px solid var(--border-color);
  gap: 8px;
}

.tab-item {
  padding: 8px 16px;
  background: var(--bg-color);
  border: 1px solid var(--border-color);
  border-bottom: none;
  border-radius: var(--radius-md) var(--radius-md) 0 0;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  font-size: 14px;
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.tab-item.active {
  background: var(--panel-bg);
  color: var(--text-primary);
  box-shadow: 0 -2px 4px rgba(0,0,0,0.02);
}

.tab-item:hover:not(.active) {
  background: #f1f5f9;
}

.tab-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--border-color);
}

.tab-item.active .tab-indicator {
  background: var(--accent-color);
}

.add-tab-btn {
  background: transparent;
  border: none;
  font-size: 20px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease;
}

.add-tab-btn:hover {
  background: var(--border-color);
}
```

Create `src/components/TabBar.tsx`:
```tsx
import React from 'react';
import '../styles/TabBar.css';

interface TabBarProps {
  tabs: string[];
  activeTab: string;
  onTabChange: (tab: string) => void;
  onAddTab: () => void;
}

export default function TabBar({ tabs, activeTab, onTabChange, onAddTab }: TabBarProps) {
  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <div 
          key={tab} 
          className={`tab-item ${activeTab === tab ? 'active' : ''}`}
          onClick={() => onTabChange(tab)}
        >
          <span className="tab-indicator"></span>
          {tab}
        </div>
      ))}
      <button className="add-tab-btn" aria-label="+" onClick={onAddTab}>+</button>
    </div>
  );
}
```

Modify `src/App.tsx`:
```tsx
import { useState } from 'react';
import './styles/App.css';
import TabBar from './components/TabBar';

function App() {
  const [tabs] = useState(['Slave 2']);
  const [activeTab, setActiveTab] = useState('Slave 2');

  return (
    <div data-testid="app-container" className="app-layout">
      <TabBar 
        tabs={tabs} 
        activeTab={activeTab} 
        onTabChange={setActiveTab} 
        onAddTab={() => {}} 
      />
    </div>
  );
}

export default App;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add TabBar navigation"
```

### Task 3: Action Toolbar Component

**Files:**
- Create: `src/components/Toolbar.tsx`
- Create: `src/styles/Toolbar.css`
- Create: `tests/Toolbar.test.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Produces: `<Toolbar />` component.

- [ ] **Step 1: Write the failing test**

Create `tests/Toolbar.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import Toolbar from '../src/components/Toolbar';

describe('Toolbar Component', () => {
  it('renders connection info and action buttons', () => {
    render(<Toolbar role="Slave" address="TCP 0.0.0.0:502" />);
    expect(screen.getByText('Slave')).toBeInTheDocument();
    expect(screen.getByText('TCP 0.0.0.0:502')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Connect' })).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Create `src/styles/Toolbar.css`:
```css
.toolbar {
  display: flex;
  align-items: center;
  padding: 16px;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border-color);
  justify-content: space-between;
}

.toolbar-left, .toolbar-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.role-text {
  font-weight: 600;
  font-size: 16px;
}

.separator {
  color: var(--border-color);
}

.address-text {
  font-family: monospace;
  font-weight: 600;
  font-size: 14px;
}

.btn {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  background: var(--panel-bg);
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn:hover {
  background: var(--bg-color);
  border-color: #d1d5db;
}

.btn-primary {
  border-color: var(--text-primary);
  color: var(--text-primary);
}

.btn-primary:hover {
  background: var(--text-primary);
  color: white;
}
```

Create `src/components/Toolbar.tsx`:
```tsx
import React from 'react';
import '../styles/Toolbar.css';

interface ToolbarProps {
  role: string;
  address: string;
}

export default function Toolbar({ role, address }: ToolbarProps) {
  return (
    <div className="toolbar">
      <div className="toolbar-left">
        <span className="role-text">{role}</span>
        <span className="separator">•</span>
        <span className="address-text">{address}</span>
        <button className="btn btn-primary">Connect</button>
        <button className="btn">Settings</button>
      </div>
      <div className="toolbar-right">
        <button className="btn">Random</button>
        <button className="btn">Increment</button>
      </div>
    </div>
  );
}
```

Modify `src/App.tsx`:
```tsx
import { useState } from 'react';
import './styles/App.css';
import TabBar from './components/TabBar';
import Toolbar from './components/Toolbar';

function App() {
  const [tabs] = useState(['Slave 2']);
  const [activeTab, setActiveTab] = useState('Slave 2');

  return (
    <div data-testid="app-container" className="app-layout">
      <TabBar tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} onAddTab={() => {}} />
      <Toolbar role="Slave" address="TCP 0.0.0.0:502" />
    </div>
  );
}

export default App;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add Toolbar component"
```

### Task 4: Configuration Row Component

**Files:**
- Create: `src/components/ConfigRow.tsx`
- Create: `src/styles/ConfigRow.css`
- Create: `tests/ConfigRow.test.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Produces: `<ConfigRow />` component.

- [ ] **Step 1: Write the failing test**

Create `tests/ConfigRow.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import ConfigRow from '../src/components/ConfigRow';

describe('ConfigRow Component', () => {
  it('renders configuration inputs', () => {
    render(<ConfigRow />);
    expect(screen.getByLabelText(/Unit ID/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Function/i)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Create `src/styles/ConfigRow.css`:
```css
.config-row {
  display: flex;
  gap: 16px;
  padding: 16px;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border-color);
  align-items: flex-end;
  flex-wrap: wrap;
}

.input-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.input-group label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.input-group input, .input-group select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 14px;
  font-family: var(--font-family);
  background: white;
  min-width: 80px;
  outline: none;
  transition: border-color 0.2s;
}

.input-group input:focus, .input-group select:focus {
  border-color: var(--accent-color);
}

.toggle-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
  margin-left: auto;
}

.toggle-switch {
  width: 40px;
  height: 20px;
  background: var(--border-color);
  border-radius: 10px;
  position: relative;
  cursor: pointer;
}

.toggle-knob {
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
  box-shadow: 0 1px 3px rgba(0,0,0,0.2);
}
```

Create `src/components/ConfigRow.tsx`:
```tsx
import React from 'react';
import '../styles/ConfigRow.css';

export default function ConfigRow() {
  return (
    <div className="config-row">
      <div className="input-group">
        <label htmlFor="unitId">Unit ID</label>
        <input id="unitId" type="number" defaultValue={1} />
      </div>
      <div className="input-group">
        <label htmlFor="functionCode">Function</label>
        <select id="functionCode">
          <option>Holding (0x03)</option>
        </select>
      </div>
      <div className="input-group">
        <label htmlFor="startAddr">Start</label>
        <input id="startAddr" type="number" defaultValue={0} />
      </div>
      <div className="input-group">
        <label htmlFor="count">Count</label>
        <input id="count" type="number" defaultValue={40} />
      </div>
      <div className="input-group">
        <label htmlFor="dataType">Data Type</label>
        <select id="dataType">
          <option>Int16</option>
        </select>
      </div>
      <div className="input-group">
        <label htmlFor="format">Format</label>
        <select id="format">
          <option>Dec</option>
        </select>
      </div>
      <div className="input-group">
        <label htmlFor="byteOrder">Byte Order</label>
        <select id="byteOrder">
          <option>ABCD</option>
        </select>
      </div>
      <div className="input-group">
        <label htmlFor="interval">Interval</label>
        <input id="interval" type="number" defaultValue={1000} />
      </div>
      <div className="toggle-group">
        <label>Raw</label>
        <div className="toggle-switch">
          <div className="toggle-knob"></div>
        </div>
      </div>
    </div>
  );
}
```

Modify `src/App.tsx`:
```tsx
import { useState } from 'react';
import './styles/App.css';
import TabBar from './components/TabBar';
import Toolbar from './components/Toolbar';
import ConfigRow from './components/ConfigRow';

function App() {
  const [tabs] = useState(['Slave 2']);
  const [activeTab, setActiveTab] = useState('Slave 2');

  return (
    <div data-testid="app-container" className="app-layout">
      <TabBar tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} onAddTab={() => {}} />
      <Toolbar role="Slave" address="TCP 0.0.0.0:502" />
      <ConfigRow />
    </div>
  );
}

export default App;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add ConfigRow component"
```

### Task 5: Data Grid Component

**Files:**
- Create: `src/components/DataGrid.tsx`
- Create: `src/styles/DataGrid.css`
- Create: `tests/DataGrid.test.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Produces: `<DataGrid />` matrix of addresses and values.

- [ ] **Step 1: Write the failing test**

Create `tests/DataGrid.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import DataGrid from '../src/components/DataGrid';

describe('DataGrid Component', () => {
  it('renders grid headers and cells', () => {
    render(<DataGrid />);
    expect(screen.getByText('Address')).toBeInTheDocument();
    expect(screen.getByText('10')).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Create `src/styles/DataGrid.css`:
```css
.data-grid-container {
  flex: 1;
  padding: 16px;
  overflow: auto;
  background: var(--bg-color);
}

.data-grid {
  width: 100%;
  border-collapse: collapse;
  background: var(--panel-bg);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
}

.data-grid th, .data-grid td {
  border: 1px solid var(--border-color);
  padding: 12px;
  text-align: center;
  font-size: 14px;
}

.data-grid th {
  background: #f9fafb;
  font-weight: 600;
  color: var(--text-primary);
}

.data-grid td.address-col {
  font-weight: 600;
  background: #f9fafb;
}

.data-grid td.value-col {
  font-family: monospace;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.2s;
}

.data-grid td.value-col:hover {
  background: #f1f5f9;
  color: var(--text-primary);
}
```

Create `src/components/DataGrid.tsx`:
```tsx
import React from 'react';
import '../styles/DataGrid.css';

export default function DataGrid() {
  const rows = [0, 10, 20, 30];
  const cols = Array.from({ length: 10 }, (_, i) => i);

  return (
    <div className="data-grid-container">
      <table className="data-grid">
        <thead>
          <tr>
            <th>Address</th>
            {cols.map(col => <th key={`header-${col}`}>{col}</th>)}
          </tr>
        </thead>
        <tbody>
          {rows.map(row => (
            <tr key={`row-${row}`}>
              <td className="address-col">{row}</td>
              {cols.map(col => (
                <td key={`cell-${row}-${col}`} className="value-col">0</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
```

Modify `src/App.tsx`:
```tsx
import { useState } from 'react';
import './styles/App.css';
import TabBar from './components/TabBar';
import Toolbar from './components/Toolbar';
import ConfigRow from './components/ConfigRow';
import DataGrid from './components/DataGrid';

function App() {
  const [tabs] = useState(['Slave 2']);
  const [activeTab, setActiveTab] = useState('Slave 2');

  return (
    <div data-testid="app-container" className="app-layout">
      <TabBar tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} onAddTab={() => {}} />
      <Toolbar role="Slave" address="TCP 0.0.0.0:502" />
      <ConfigRow />
      <DataGrid />
    </div>
  );
}

export default App;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add DataGrid component"
```

### Task 6: Status Footer Component

**Files:**
- Create: `src/components/StatusBar.tsx`
- Create: `src/styles/StatusBar.css`
- Create: `tests/StatusBar.test.tsx`
- Modify: `src/App.tsx`

**Interfaces:**
- Produces: `<StatusBar />` for application status.

- [ ] **Step 1: Write the failing test**

Create `tests/StatusBar.test.tsx`:
```tsx
import { render, screen } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import StatusBar from '../src/components/StatusBar';

describe('StatusBar Component', () => {
  it('renders status message and copyright', () => {
    render(<StatusBar message="Connect before writing." status="error" />);
    expect(screen.getByText(/Connect before writing/i)).toBeInTheDocument();
    expect(screen.getByText(/© Modlab/i)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Create `src/styles/StatusBar.css`:
```css
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: var(--panel-bg);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.error { background: var(--danger-color); }
.status-dot.success { background: #10b981; }
.status-dot.idle { background: var(--text-secondary); }

.status-message {
  font-weight: 500;
}

.status-right {
  color: var(--text-secondary);
}
```

Create `src/components/StatusBar.tsx`:
```tsx
import React from 'react';
import '../styles/StatusBar.css';

interface StatusBarProps {
  message: string;
  status: 'error' | 'success' | 'idle';
}

export default function StatusBar({ message, status }: StatusBarProps) {
  return (
    <div className="status-bar">
      <div className="status-left">
        <div className={`status-dot ${status}`}></div>
        <span className="status-message" style={{ color: status === 'error' ? 'var(--danger-color)' : 'inherit' }}>
          {message}
        </span>
      </div>
      <div className="status-right">
        © Modlab • dote27@163.com
      </div>
    </div>
  );
}
```

Modify `src/App.tsx`:
```tsx
import { useState } from 'react';
import './styles/App.css';
import TabBar from './components/TabBar';
import Toolbar from './components/Toolbar';
import ConfigRow from './components/ConfigRow';
import DataGrid from './components/DataGrid';
import StatusBar from './components/StatusBar';

function App() {
  const [tabs] = useState(['Slave 2']);
  const [activeTab, setActiveTab] = useState('Slave 2');

  return (
    <div data-testid="app-container" className="app-layout">
      <TabBar tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} onAddTab={() => {}} />
      <Toolbar role="Slave" address="TCP 0.0.0.0:502" />
      <ConfigRow />
      <DataGrid />
      <StatusBar message="[Slave 2] Connect before writing." status="error" />
    </div>
  );
}

export default App;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "feat: add StatusBar and finalize layout"
```
