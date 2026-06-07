/**
 * Shared IPC contracts between Echo processes.
 * Phase 0 placeholder — JSON-RPC envelope types land in Phase 1.
 */
export type EchoSubsystem = "voice" | "brain" | "skills" | "memory" | "hud";

export interface SubsystemHealth {
  subsystem: EchoSubsystem;
  status: "green" | "yellow" | "red";
}
