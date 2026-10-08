import { Component, type ReactNode } from 'react';
import { messages as t } from '../shared/i18n';
import { Button } from '../shared/ui/button';
export class ErrorBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  render() {
    if (this.state.failed) return <main className="fatal-state" role="alert"><h1>{t.errorBoundary}</h1><Button onClick={() => window.location.reload()}>{t.reload}</Button></main>;
    return this.props.children;
  }
}
