import { AppShell } from './app/AppShell';
import { ErrorBoundary } from './app/ErrorBoundary';
import './App.css';
export default function App() { return <ErrorBoundary><AppShell /></ErrorBoundary>; }
