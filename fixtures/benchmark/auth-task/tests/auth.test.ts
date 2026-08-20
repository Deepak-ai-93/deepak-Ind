import { authenticate } from "../src/auth.js";

if (!authenticate("fixture-token")) throw new Error("auth fixture failed");
