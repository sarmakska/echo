import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { echoTurn } from "../lib/turn";
import { CommandBar } from "./CommandBar";

vi.mock("../lib/turn", () => ({ echoTurn: vi.fn() }));
const mockAsk = echoTurn as unknown as ReturnType<typeof vi.fn>;

describe("CommandBar", () => {
  beforeEach(() => mockAsk.mockReset());

  it("submits the prompt and reports the brain reply", async () => {
    mockAsk.mockResolvedValue("Sunny in Hemel.");
    const onReply = vi.fn();
    const user = userEvent.setup();
    render(<CommandBar onReply={onReply} />);

    await user.type(screen.getByLabelText("command"), "weather");
    await user.click(screen.getByRole("button", { name: /ask/i }));

    await waitFor(() => expect(onReply).toHaveBeenCalledWith("Sunny in Hemel."));
    expect(mockAsk).toHaveBeenCalledWith("weather");
  });

  it("passes through a failure message returned by askBrain", async () => {
    mockAsk.mockResolvedValue("Echo couldn't reach the brain: claude not found");
    const onReply = vi.fn();
    const user = userEvent.setup();
    render(<CommandBar onReply={onReply} />);

    await user.type(screen.getByLabelText("command"), "hi");
    await user.click(screen.getByRole("button", { name: /ask/i }));

    await waitFor(() =>
      expect(onReply).toHaveBeenCalledWith(expect.stringContaining("claude not found")),
    );
  });

  it("ignores empty submissions", async () => {
    const user = userEvent.setup();
    render(<CommandBar />);
    await user.click(screen.getByRole("button", { name: /ask/i }));
    expect(mockAsk).not.toHaveBeenCalled();
  });
});
