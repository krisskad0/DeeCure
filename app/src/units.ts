// SPDX-License-Identifier: Apache-2.0
// Units and refresh choices from Settings › General.
// Temperatures are measured in °C everywhere; only what is shown changes. Warning
// limits (such as 90 °C) always compare Celsius values.

import { readPref } from "./prefs";

export type TempUnit = "celsius" | "fahrenheit";
export type RefreshChoice = "2s" | "5s" | "15s" | "manual";

/** The live-refresh interval in ms, or `null` for manual. */
export function refreshMs(choice: RefreshChoice): number | null {
  return choice === "manual" ? null : { "2s": 2_000, "5s": 5_000, "15s": 15_000 }[choice];
}

const unit = (): TempUnit => readPref<TempUnit>("temp", "celsius");

/** A Celsius value in the chosen unit. */
export function tempValue(celsius: number): number {
  return unit() === "fahrenheit" ? (celsius * 9) / 5 + 32 : celsius;
}

/** `°C` or `°F`. */
export function tempUnit(): string {
  return unit() === "fahrenheit" ? "°F" : "°C";
}

/** A Celsius value shown in the chosen unit, such as `54 °C` or `129 °F`. */
export function temp(celsius: number): string {
  return `${Math.round(tempValue(celsius))} ${tempUnit()}`;
}

/** Sent when the Refresh button is pressed: every live panel loads again now. */
export const REFRESH_EVENT = "deecure-refresh";
