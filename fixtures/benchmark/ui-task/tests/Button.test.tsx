import { Button } from "../src/Button.js";

if (!Button("IND").includes("primary")) throw new Error("button fixture failed");
