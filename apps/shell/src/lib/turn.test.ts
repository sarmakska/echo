import { invoke } from "@tauri-apps/api/core";
import { echoTurn, episodeDay } from "./turn";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("episodeDay", () => {
  it("formats as zero-padded YYYY/MM/DD", () => {
    // Month is 0-indexed in JS Date; March 7th 2026.
    expect(episodeDay(new Date(2026, 2, 7))).toBe("2026/03/07");
  });
});

describe("echoTurn", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("invokes echo_turn with prompt, day and ts", async () => {
    mockInvoke.mockResolvedValue("Standup at 9:30.");
    const now = new Date(2026, 5, 7, 9, 0, 0);
    const reply = await echoTurn("what is on today", now);

    expect(reply).toBe("Standup at 9:30.");
    expect(mockInvoke).toHaveBeenCalledWith("echo_turn", {
      prompt: "what is on today",
      day: "2026/06/07",
      ts: now.toISOString(),
    });
  });
});
