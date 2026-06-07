import { render, screen } from "@testing-library/react";
import { HudCard } from "./HudCard";

describe("HudCard", () => {
  it("renders the echo wordmark", () => {
    render(<HudCard />);
    expect(screen.getByText(/echo/i)).toBeInTheDocument();
  });

  it("renders an idle status label by default", () => {
    render(<HudCard />);
    expect(screen.getByText(/idle/i)).toBeInTheDocument();
  });
});
