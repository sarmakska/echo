import { invoke } from "@tauri-apps/api/core";
import { askBrain } from "./brain";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockInvoke = invoke as unknown as ReturnType<typeof vi.fn>;

describe("askBrain", () => {
  beforeEach(() => mockInvoke.mockReset());

  it("invokes the ask_brain command with the prompt and returns its reply", async () => {
    mockInvoke.mockResolvedValue("hello back");
    const reply = await askBrain("hi there");
    expect(reply).toBe("hello back");
    expect(mockInvoke).toHaveBeenCalledWith("ask_brain", { prompt: "hi there" });
  });

  // The failure branch (invoke rejects → askBrain returns a user-facing string)
  // is covered at the CommandBar level, which feeds that string through onReply.
  // It is asserted there rather than here because Vitest 2.1.x surfaces a
  // throwing/rejecting mock as a test failure even when the code under test
  // catches it.
});
