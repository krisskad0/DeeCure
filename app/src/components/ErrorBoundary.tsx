// SPDX-License-Identifier: Apache-2.0
// Keeps one broken part from breaking the page: when something inside fails to draw, only that
// part shows a short message with the reason and a "Try again" button. The rest keeps working.

import { Component, type ReactNode } from "react";
import { Button } from "./Button";

interface Props {
  /** Where the failure is shown: inside a panel body, or as a grid item of its own. */
  inline?: boolean;
  /** A short name for what failed, such as "Processor details". */
  what?: string;
  /** Changing it clears the error, such as the page and tab. */
  resetKey?: string;
  children: ReactNode;
}

interface State {
  error: Error | null;
  key?: string;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null, key: this.props.resetKey };

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { error };
  }

  static getDerivedStateFromProps(props: Props, state: State): Partial<State> | null {
    return props.resetKey !== state.key ? { error: null, key: props.resetKey } : null;
  }

  componentDidCatch(error: Error) {
    // Kept for the developer console; the person sees the message below.
    console.warn(`DeeCure: ${this.props.what ?? "a part of the page"} could not be shown:`, error);
  }

  render() {
    const { error } = this.state;
    if (!error) return this.props.children;
    const body = (
      <div className="empty muted" role="alert">
        {this.props.what ?? "This part"} could not be shown ({error.message || "unknown error"}).{" "}
        <Button small onClick={() => this.setState({ error: null })}>
          Try again
        </Button>
      </div>
    );
    return this.props.inline ? (
      body
    ) : (
      <section className="panel span-12">
        <div className="panel-body">{body}</div>
      </section>
    );
  }
}

/** An Expert section with its own boundary, so a failure there leaves the rest of the tab. */
export function Guard({ what, children }: { what?: string; children: ReactNode }) {
  return <ErrorBoundary what={what ?? "This Expert section"}>{children}</ErrorBoundary>;
}
