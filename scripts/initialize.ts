import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HotCrown } from "../target/types/hot_crown";
import {
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.HotCrown as Program<HotCrown>;
  const admin = provider.wallet;

  const tokenMint = new anchor.web3.PublicKey(
    "8gKUgdkSGMqQMgCxnMQEDV19Eb3riysKgh9MvbEDiNhf"
  );

  // Derive PDA
  const [gameStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("game_state")],
    program.programId
  );

  // Throne vault ATA (owned by PDA)
  const throneVault = await getAssociatedTokenAddress(
    tokenMint,
    gameStatePda,
    true
  );

  // Dev wallet ATA (admin's own ATA)
  const devWalletAta = await getAssociatedTokenAddress(
    tokenMint,
    admin.publicKey
  );

  console.log("Program ID:", program.programId.toBase58());
  console.log("Admin:", admin.publicKey.toBase58());
  console.log("Token Mint:", tokenMint.toBase58());
  console.log("Game State PDA:", gameStatePda.toBase58());
  console.log("Throne Vault:", throneVault.toBase58());
  console.log("Dev Wallet ATA:", devWalletAta.toBase58());

  console.log("\nInitializing game...");

  const tx = await program.methods
    .initialize()
    .accountsStrict({
      admin: admin.publicKey,
      gameState: gameStatePda,
      tokenMint,
      throneVault,
      devWalletAta,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("Transaction:", tx);
  console.log("Game initialized successfully!");

  const state = await program.account.gameState.fetch(gameStatePda);
  console.log("\nGame State:");
  console.log("  Phase:", JSON.stringify(state.phase));
  console.log("  Admin:", state.admin.toBase58());
  console.log("  Token Mint:", state.tokenMint.toBase58());
  console.log("  Next Bid:", state.nextBidAmount.toNumber());
  console.log("  Paused:", state.paused);
}

main().catch(console.error);
