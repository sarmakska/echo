import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SetupWizard } from "./SetupWizard";

describe("SetupWizard", () => {
  it("walks through all four steps and completes with the chosen config", async () => {
    const user = userEvent.setup();
    const onComplete = vi.fn();
    render(<SetupWizard onComplete={onComplete} />);

    // Step 1: brain — pick Gemini.
    await user.click(screen.getByRole("radio", { name: /gemini/i }));
    await user.click(screen.getByRole("button", { name: /next/i }));

    // Step 2: skills — drop web search, keep weather + files.
    await user.click(screen.getByRole("checkbox", { name: /web search/i }));
    await user.click(screen.getByRole("button", { name: /next/i }));

    // Step 3: wake word — change it.
    const wake = screen.getByLabelText("wake word");
    await user.clear(wake);
    await user.type(wake, "Jarvis");
    await user.click(screen.getByRole("button", { name: /next/i }));

    // Step 4: voice — finish.
    await user.click(screen.getByRole("button", { name: /finish/i }));

    expect(onComplete).toHaveBeenCalledWith({
      brain: "gemini",
      skills: ["weather", "files-local"],
      wakeWord: "Jarvis",
      voice: "British female",
    });
  });

  it("defaults to claude with all three skills connected", async () => {
    const user = userEvent.setup();
    const onComplete = vi.fn();
    render(<SetupWizard onComplete={onComplete} />);

    await user.click(screen.getByRole("button", { name: /next/i })); // brain
    await user.click(screen.getByRole("button", { name: /next/i })); // skills
    await user.click(screen.getByRole("button", { name: /next/i })); // wake
    await user.click(screen.getByRole("button", { name: /finish/i }));

    expect(onComplete).toHaveBeenCalledWith({
      brain: "claude",
      skills: ["weather", "web-search", "files-local"],
      wakeWord: "Echo",
      voice: "British female",
    });
  });
});
