import { Component, type ReactNode } from 'react';

/** Recovery content is best effort if application state also breaks the old page. */
export class RecoveryBoundary extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  render() { return this.state.failed ? null : this.props.children; }
}

interface NavigationBoundaryProps {
  sequence: number;
  children: ReactNode;
  fallback: () => ReactNode;
  onFailure: () => void;
  onRetry: () => void;
  restoring: boolean;
}

/** A failed candidate never runs its successful-page commit effect. */
export class NavigationBoundary extends Component<NavigationBoundaryProps, { sequence: number; failed: boolean }> {
  state = { sequence: this.props.sequence, failed: false };
  static getDerivedStateFromProps(props: NavigationBoundaryProps, state: { sequence: number }) {
    return props.sequence === state.sequence ? null : { sequence: props.sequence, failed: false };
  }
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch() { this.props.onFailure(); }
  render() {
    if (!this.state.failed) return this.props.children;
    return <>{this.props.fallback()}<aside role="alert">
      <p>Could not open this page. You can retry the navigation.</p>
      <button type="button" disabled={this.props.restoring} onClick={this.props.onRetry}>Retry navigation</button>
    </aside></>;
  }
}
