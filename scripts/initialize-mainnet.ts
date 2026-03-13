import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HotCrown } from "../target/types/hot_crown";
import {
  getAssociatedTokenAddress,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = new Program(
    require("../target/idl/hot_crown.json"),
    provider
  );
  const admin = provider.wallet;

  const tokenMint = new PublicKey(
    "3JPJhu1LJZnyXniSVLkrJqYBrqzhw5oxo8TZtJg8pump"
  );

  const devWalletAta = new PublicKey(
    "CnW61obKZSaNi3dY6sxqwpzgJLxCd7SxrScH6xPERBE7"
  );

  // Derive PDA
  const [gameStatePda] = PublicKey.findProgramAddressSync(
    [Buffer.from("game_state")],
    program.programId
  );

  // Throne vault ATA (owned by PDA, Token-2022)
  const throneVault = await getAssociatedTokenAddress(
    tokenMint,
    gameStatePda,
    true,
    TOKEN_2022_PROGRAM_ID
  );

  console.log("Program ID:", program.programId.toBase58());
  console.log("Admin:", admin.publicKey.toBase58());
  console.log("Token Mint:", tokenMint.toBase58());
  console.log("Game State PDA:", gameStatePda.toBase58());
  console.log("Throne Vault:", throneVault.toBase58());
  console.log("Dev Wallet ATA:", devWalletAta.toBase58());

  console.log("\nInitializing game on MAINNET...");

  const tx = await program.methods
    .initialize()
    .accountsStrict({
      admin: admin.publicKey,
      gameState: gameStatePda,
      tokenMint,
      throneVault,
      devWalletAta,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("Transaction:", tx);
  console.log("Game initialized successfully on MAINNET!");

  const state = await (program.account as any).gameState.fetch(gameStatePda);
  console.log("\nGame State:");
  console.log("  Phase:", JSON.stringify(state.phase));
  console.log("  Admin:", state.admin.toBase58());
  console.log("  Token Mint:", state.tokenMint.toBase58());
  console.log("  Next Bid:", state.nextBidAmount.toNumber());
  console.log("  Paused:", state.paused);
}

main().catch(console.error);
