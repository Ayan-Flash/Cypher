import type { Command } from "@/command";
import THREATS from "./threats.txt";

export function threatsCommand(): Command.Info {
  return {
    name: "threats",
    description: "perform a threat model scan and attacker path analysis",
    template: THREATS,
    hints: ["$ARGUMENTS"],
  };
}
