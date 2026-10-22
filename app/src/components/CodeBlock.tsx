// SPDX-License-Identifier: Apache-2.0
// 22 Code and log blocks (docs/03-ui-design/03-components/22-code-and-log-blocks.md).

export function Command({ children }: { children: string }) {
  return <pre className="cmd">{children}</pre>;
}

export function LogLine({ children }: { children: string }) {
  return <pre className="log">{children}</pre>;
}
