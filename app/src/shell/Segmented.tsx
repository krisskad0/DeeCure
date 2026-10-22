// SPDX-License-Identifier: Apache-2.0
// Segmented control (docs/03-ui-design/02-shell/top-bar.md).

export function Segmented<T extends string>({
  label,
  options,
  value,
  onChange,
  className = "",
}: {
  label: string;
  options: readonly { value: T; label: string }[];
  value: T;
  onChange: (value: T) => void;
  className?: string;
}) {
  return (
    <div className={`seg ${className}`} role="group" aria-label={label}>
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          aria-pressed={option.value === value}
          onClick={() => onChange(option.value)}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
