export type WhatPresetId = "any" | "text" | "json" | "bytes";
export type HowPresetId = "single" | "stream" | "request-response" | "broadcast";

export interface PortDeclaration {
  what: { preset: WhatPresetId | "custom"; id?: string };
  how: { preset: HowPresetId | "custom"; id?: string };
}

export type Axis = "What" | "How";

export interface ConnectionValidation {
  compatible: boolean;
  axis?: Axis;
  reason?: string;
  hint?: string;
}

export function whatId(decl: PortDeclaration): string {
  return decl.what.preset === "custom" ? "custom" : decl.what.preset;
}

export function howId(decl: PortDeclaration): string {
  return decl.how.preset === "custom" ? "custom" : decl.how.preset;
}

export function portLabel(decl: PortDeclaration): string {
  return `${whatId(decl)}·${howId(decl)}`;
}

/** Mirrors Rust resolve_ports for sync drag validation. */
export function resolvePorts(
  source: PortDeclaration,
  target: PortDeclaration,
): ConnectionValidation {
  const what = resolveWhat(whatId(source), whatId(target));
  if (!what.compatible) return what;
  return resolveHow(howId(source), howId(target));
}

function resolveWhat(source: string, target: string): ConnectionValidation {
  if (source === "custom" || target === "custom") {
    return reject("What", "Custom What presets are not supported in v3.", "Use any, text, json, or bytes.");
  }
  if (source === "any" || target === "any" || source === target) {
    return { compatible: true };
  }
  return reject(
    "What",
    `What mismatch: source ${source} cannot connect to target ${target}.`,
    `Change the target port to ${source} or the source port to ${target}.`,
  );
}

function resolveHow(source: string, target: string): ConnectionValidation {
  if (source === "custom" || target === "custom") {
    return reject("How", "Custom How presets are not supported in v3.", "Use single, stream, request-response, or broadcast.");
  }
  if (source === "single" && target === "single") {
    return { compatible: true };
  }
  return reject(
    "How",
    `How mismatch: ${source} → ${target} is not supported yet.`,
    "Only single → single connections are supported in v3 (channels in v10, adapters in v13).",
  );
}

function reject(axis: Axis, reason: string, hint: string): ConnectionValidation {
  return { compatible: false, axis, reason, hint };
}

export function formatRejection(validation: ConnectionValidation): string {
  if (validation.compatible) return "";
  const axis = validation.axis ? `Rejected (${validation.axis})` : "Rejected";
  const reason = validation.reason ?? "Connection not allowed";
  const hint = validation.hint ? ` — Hint: ${validation.hint}` : "";
  return `${axis}: ${reason}${hint}`;
}

export const WHAT_OPTIONS: WhatPresetId[] = ["any", "text", "json", "bytes"];
export const HOW_OPTIONS: HowPresetId[] = ["single", "stream", "request-response", "broadcast"];
