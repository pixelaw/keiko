import {Provider} from "starknet";

export const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000"
export const PROVIDER = new Provider({
  rpc: {
    nodeUrl: import.meta.env.PUBLIC_NODE_URL
  }
});