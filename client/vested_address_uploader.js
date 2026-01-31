import { ApiPromise, WsProvider } from "@polkadot/api";
import { ContractPromise } from "@polkadot/api-contract";
import { Keyring } from "@polkadot/keyring";
import fs from "fs";
import 'dotenv/config';
import { decode } from "./decode.js";
import csv from "csv-parser";

/// Blockchain
const WS_ENDPOINT = process.env.WS_ENDPOINT;
const CONTRACT_ADDRESS = process.env.CONTRACT_ADDRESS;
const CONTRACT_ABI_PATH = process.env.CONTRACT_ABI_PATH;
const OWNER = process.env.OWNER;

console.log("Connecting to blockchain...");
const wsProvider = new WsProvider(WS_ENDPOINT);
const api = await ApiPromise.create({ provider: wsProvider });
console.log("Connected to:", (await api.rpc.system.chain()).toHuman());

/// Contract
const abiJSON = JSON.parse(fs.readFileSync(CONTRACT_ABI_PATH, "utf8"));
const contract = new ContractPromise(api, abiJSON, CONTRACT_ADDRESS);
const gasLimit = api.registry.createType('WeightV2', {
          refTime: 300000000000,
          proofSize: 500000,
});
const storageDepositLimit = null;

/// Reading and processing the CSV file
const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
///const stream = fs.createReadStream("./live/vested_address2.csv").pipe(csv());
///const stream = fs.createReadStream("./live/vested_address3.csv").pipe(csv());
const stream = fs.createReadStream("./live/vested_address4.csv").pipe(csv());
///const stream = fs.createReadStream("./live/vested_address5.csv").pipe(csv());
///const stream = fs.createReadStream("./live/vested_address6.csv").pipe(csv());
///const stream = fs.createReadStream("./live/vested_address7.csv").pipe(csv());
///const stream = fs.createReadStream("./live/vested_address8.csv").pipe(csv());
///const stream = fs.createReadStream("./live/vested_address9.csv").pipe(csv());

stream.on("data", async (data) => {
  stream.pause(); // pause stream while processing

  try {
    console.log("No:", data.No);
    console.log("Address:", data.Address);
    console.log("Balance:", data.Balance);

    // perform the operation here (add vested balance)
    const keyring = new Keyring({ type: "sr25519" });
    const owner = keyring.addFromUri(OWNER);
    await new Promise(async (resolve, reject) => {
      const unsub = await contract.tx
        .addVestedBalance({ storageDepositLimit, gasLimit }, 
          String(data.Address ?? "").trim(),
          String(data.Balance).replace(/"/g, "").trim(),
        ).signAndSend(owner, ({ status, events, dispatchError }) => {   
          console.log("Status:", status?.type);
          if(events?.length > 0) {
            events.forEach(({ event }) => {
              if (event.section === "contracts" && event.method === "ContractEmitted") {
                console.log(decode(event.data));
                unsub();
                resolve();
              }
            });
          }
      });
    });

    // 30 seconds delay
    await sleep(20000); 

  } catch (err) {
    console.error("Error processing row:", err);

  } finally {
    stream.resume(); // resume stream

  }
  
});

stream.on("end", () => {
  console.log("CSV processing completed.");
});