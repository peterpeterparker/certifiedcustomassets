import { Actor, HttpAgent } from "@dfinity/agent";

export const createActor = ({canisterId, options, factory}) => {
    const agent = new HttpAgent(options ? { ...options.agentOptions } : {});

    // Fetch root key for certificate validation during development
    if (process.env.NODE_ENV !== "production") {
        agent.fetchRootKey().catch((err) => {
            console.warn(
                "Unable to fetch root key. Check to ensure that your local replica is running"
            );
            console.error(err);
        });
    }

    // Creates an actor with using the candid interface and the HttpAgent
    return Actor.createActor(factory, {
        agent,
        canisterId,
        ...(options ? options.actorOptions : {}),
    });
};