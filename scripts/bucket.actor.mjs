import fetch from "node-fetch";
import { idlFactory } from "../src/declarations/certifiedcustomassets_backend/certifiedcustomassets_backend.did.mjs";
import { createActor } from "./actor.mjs";

const MAINNET = true;

// Production: not deploy
// local rrkah-fqaaa-aaaaa-aaaaq-cai
export const canisterId = MAINNET
  ? "okoji-rqaaa-aaaap-qasma-cai"
  : "rrkah-fqaaa-aaaaa-aaaaq-cai";

export const bucketActor = createActor({
  canisterId,
  options: {
    agentOptions: {
      fetch,
      host: MAINNET ? "https://ic0.app" : "http://localhost:8000",
    },
  },
  factory: idlFactory,
});
