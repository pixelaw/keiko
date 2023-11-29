import {Provider} from "starknet";

export const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000"

export const KATANA_URL = import.meta.env.KATANA_URL

export const PROVIDER = new Provider({
  rpc: {
    nodeUrl: KATANA_URL
  }
});