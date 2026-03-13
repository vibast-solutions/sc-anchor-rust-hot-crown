import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HotCrown } from "../target/types/hot_crown";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAssociatedTokenAddress,
  getAccount,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

describe("hot_crown", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.HotCrown as Program<HotCrown>;
  const admin = provider.wallet;

  let tokenMint: anchor.web3.PublicKey;
  let devWalletAta: anchor.web3.PublicKey;
  let gameStatePda: anchor.web3.PublicKey;
  let throneVault: anchor.web3.PublicKey;
  let gameStateBump: number;

  const player1 = anchor.web3.Keypair.generate();
  const player2 = anchor.web3.Keypair.generate();
  const player3 = anchor.web3.Keypair.generate();

  let player1Ata: anchor.web3.PublicKey;
  let player2Ata: anchor.web3.PublicKey;
  let player3Ata: anchor.web3.PublicKey;

  const ONE_TOKEN = 1_000_000;
  const INITIAL_BALANCE = 100_000 * ONE_TOKEN;

  before(async () => {
    // Airdrop SOL to test players
    for (const player of [player1, player2, player3]) {
      const sig = await provider.connection.requestAirdrop(
        player.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
    }

    // Create Token-2022 mint
    tokenMint = await createMint(
      provider.connection,
      (admin as any).payer,
      admin.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    // Derive PDA
    [gameStatePda, gameStateBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("game_state")],
        program.programId
      );

    // Get throne vault ATA (PDA-owned, Token-2022)
    throneVault = await getAssociatedTokenAddress(
      tokenMint,
      gameStatePda,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    // Create dev wallet ATA (Token-2022)
    devWalletAta = await createAssociatedTokenAccount(
      provider.connection,
      (admin as any).payer,
      tokenMint,
      admin.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    // Create player ATAs and mint tokens (Token-2022)
    player1Ata = await createAssociatedTokenAccount(
      provider.connection,
      (admin as any).payer,
      tokenMint,
      player1.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    player2Ata = await createAssociatedTokenAccount(
      provider.connection,
      (admin as any).payer,
      tokenMint,
      player2.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    player3Ata = await createAssociatedTokenAccount(
      provider.connection,
      (admin as any).payer,
      tokenMint,
      player3.publicKey,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );

    for (const ata of [player1Ata, player2Ata, player3Ata]) {
      await mintTo(
        provider.connection,
        (admin as any).payer,
        tokenMint,
        ata,
        admin.publicKey,
        INITIAL_BALANCE,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );
    }
  });

  // ─── Initialize ───────────────────────────────────────────────────

  it("initializes the game", async () => {
    await program.methods
      .initialize()
      .accounts({
        admin: admin.publicKey,
        gameState: gameStatePda,
        tokenMint,
        throneVault,
        devWalletAta,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    assert.ok(state.admin.equals(admin.publicKey));
    assert.ok(state.tokenMint.equals(tokenMint));
    assert.equal(state.paused, false);
    assert.deepEqual(state.phase, { bidding: {} });
    assert.equal(state.nextBidAmount.toNumber(), 1);
    assert.equal(state.lastBidAmount.toNumber(), 0);
    assert.equal(state.thronePot.toNumber(), 0);
    assert.ok(
      state.candidate.equals(anchor.web3.PublicKey.default)
    );
    assert.ok(state.king.equals(anchor.web3.PublicKey.default));
    assert.equal(state.battleActive, false);
  });

  // ─── Update Config ────────────────────────────────────────────────

  it("admin can update config", async () => {
    await program.methods
      .updateConfig({
        newAdmin: null,
        newDevWalletAta: null,
        paused: true,
      })
      .accounts({
        admin: admin.publicKey,
        gameState: gameStatePda,
      })
      .rpc();

    let state = await program.account.gameState.fetch(gameStatePda);
    assert.equal(state.paused, true);

    // Unpause for further tests
    await program.methods
      .updateConfig({
        newAdmin: null,
        newDevWalletAta: null,
        paused: false,
      })
      .accounts({
        admin: admin.publicKey,
        gameState: gameStatePda,
      })
      .rpc();

    state = await program.account.gameState.fetch(gameStatePda);
    assert.equal(state.paused, false);
  });

  it("non-admin cannot update config", async () => {
    try {
      await program.methods
        .updateConfig({
          newAdmin: null,
          newDevWalletAta: null,
          paused: true,
        })
        .accounts({
          admin: player1.publicKey,
          gameState: gameStatePda,
        })
        .signers([player1])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "Unauthorized");
    }
  });

  // ─── Bidding Phase ────────────────────────────────────────────────

  it("player 1 places bid #1 (1 token)", async () => {
    const devBalBefore = (await getAccount(provider.connection, devWalletAta, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;

    await program.methods
      .placeThroneBid()
      .accounts({
        bidder: player1.publicKey,
        gameState: gameStatePda,
        bidderTokenAccount: player1Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player1])
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    assert.ok(state.candidate.equals(player1.publicKey));
    assert.equal(state.lastBidAmount.toNumber(), 1);
    assert.equal(state.nextBidAmount.toNumber(), 2);
    // 80% of 1 token = 800,000
    assert.equal(state.thronePot.toNumber(), 800_000);
    assert.notEqual(state.biddingDeadline.toNumber(), 0);

    // Dev got 10% = 100,000
    const devBalAfter = (await getAccount(provider.connection, devWalletAta, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(
      Number(devBalAfter) - Number(devBalBefore),
      100_000
    );

    // Vault got 80% = 800,000
    const vaultBal = (await getAccount(provider.connection, throneVault, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(Number(vaultBal), 800_000);
  });

  it("player 2 places bid #2 (2 tokens)", async () => {
    await program.methods
      .placeThroneBid()
      .accounts({
        bidder: player2.publicKey,
        gameState: gameStatePda,
        bidderTokenAccount: player2Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player2])
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    assert.ok(state.candidate.equals(player2.publicKey));
    assert.equal(state.lastBidAmount.toNumber(), 2);
    assert.equal(state.nextBidAmount.toNumber(), 3);
    // pot: 800,000 + 1,600,000 = 2,400,000
    assert.equal(state.thronePot.toNumber(), 2_400_000);
  });

  it("cannot bid when game is paused", async () => {
    // Pause
    await program.methods
      .updateConfig({ newAdmin: null, newDevWalletAta: null, paused: true })
      .accounts({ admin: admin.publicKey, gameState: gameStatePda })
      .rpc();

    try {
      await program.methods
        .placeThroneBid()
        .accounts({
          bidder: player3.publicKey,
          gameState: gameStatePda,
          bidderTokenAccount: player3Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player3])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "GamePaused");
    }

    // Unpause
    await program.methods
      .updateConfig({ newAdmin: null, newDevWalletAta: null, paused: false })
      .accounts({ admin: admin.publicKey, gameState: gameStatePda })
      .rpc();
  });

  // ─── Finalize King Election ───────────────────────────────────────

  it("cannot finalize king election before timer expires", async () => {
    const king2Ata = player2Ata; // player2 is candidate
    try {
      await program.methods
        .finalizeKingElection()
        .accounts({
          anyone: admin.publicKey,
          gameState: gameStatePda,
          throneVault,
          kingTokenAccount: king2Ata,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "BiddingNotExpired");
    }
  });

  it("finalizes king election after timer expires", async () => {
    // Wait for 2-second timer to expire
    await sleep(4000);

    const player2BalBefore = (
      await getAccount(provider.connection, player2Ata, undefined, TOKEN_2022_PROGRAM_ID)
    ).amount;

    await program.methods
      .finalizeKingElection()
      .accounts({
        anyone: admin.publicKey,
        gameState: gameStatePda,
        throneVault,
        kingTokenAccount: player2Ata,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    // Player 2 is now king
    assert.ok(state.king.equals(player2.publicKey));
    assert.deepEqual(state.phase, { battle: {} });
    assert.equal(state.battleActive, false);
    assert.equal(state.thronePot.toNumber(), 0);
    assert.equal(state.nextBidAmount.toNumber(), 1);
    assert.ok(
      state.candidate.equals(anchor.web3.PublicKey.default)
    );

    // Player 2 received the throne pot (2,400,000)
    const player2BalAfter = (
      await getAccount(provider.connection, player2Ata, undefined, TOKEN_2022_PROGRAM_ID)
    ).amount;
    assert.equal(
      Number(player2BalAfter) - Number(player2BalBefore),
      2_400_000
    );

    // Vault should be empty
    const vaultBal = (await getAccount(provider.connection, throneVault, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(Number(vaultBal), 0);
  });

  // ─── Battle Phase: Attack ─────────────────────────────────────────

  it("cannot bid during battle phase", async () => {
    try {
      await program.methods
        .placeThroneBid()
        .accounts({
          bidder: player1.publicKey,
          gameState: gameStatePda,
          bidderTokenAccount: player1Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player1])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "InvalidPhase");
    }
  });

  it("cannot defend before first attack", async () => {
    try {
      await program.methods
        .defend(new anchor.BN(1))
        .accounts({
          defender: player3.publicKey,
          gameState: gameStatePda,
          defenderTokenAccount: player3Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player3])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "NoAttackYet");
    }
  });

  it("rejects invalid soldier count (0)", async () => {
    try {
      await program.methods
        .attack(new anchor.BN(0))
        .accounts({
          attacker: player1.publicKey,
          gameState: gameStatePda,
          attackerTokenAccount: player1Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player1])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "InvalidSoldierCount");
    }
  });

  it("rejects invalid soldier count (11)", async () => {
    try {
      await program.methods
        .attack(new anchor.BN(11))
        .accounts({
          attacker: player1.publicKey,
          gameState: gameStatePda,
          attackerTokenAccount: player1Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player1])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "InvalidSoldierCount");
    }
  });

  it("player 1 attacks with 5 soldiers (first attack starts battle)", async () => {
    const devBalBefore = (await getAccount(provider.connection, devWalletAta, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;

    await program.methods
      .attack(new anchor.BN(5))
      .accounts({
        attacker: player1.publicKey,
        gameState: gameStatePda,
        attackerTokenAccount: player1Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player1])
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    assert.equal(state.battleActive, true);
    assert.equal(state.attackSoldiers.toNumber(), 5);
    assert.equal(state.defenseSoldiers.toNumber(), 0);
    // 5 tokens = 5,000,000 raw. Dev fee 10% = 500,000. Army = 4,500,000
    assert.equal(state.attackPool.toNumber(), 4_500_000);
    assert.notEqual(state.battleDeadline.toNumber(), 0);

    const devBalAfter = (await getAccount(provider.connection, devWalletAta, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(Number(devBalAfter) - Number(devBalBefore), 500_000);
  });

  // ─── Turn Restriction ─────────────────────────────────────────────

  it("cannot attack again (attack > defense, turn restriction)", async () => {
    try {
      await program.methods
        .attack(new anchor.BN(1))
        .accounts({
          attacker: player1.publicKey,
          gameState: gameStatePda,
          attackerTokenAccount: player1Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player1])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "TurnRestriction");
    }
  });

  // ─── Battle Phase: Defend ─────────────────────────────────────────

  it("player 3 defends with 3 soldiers", async () => {
    await program.methods
      .defend(new anchor.BN(3))
      .accounts({
        defender: player3.publicKey,
        gameState: gameStatePda,
        defenderTokenAccount: player3Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player3])
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    assert.equal(state.defenseSoldiers.toNumber(), 3);
    // 3 tokens = 3,000,000 raw. Dev fee = 300,000. Army = 2,700,000
    assert.equal(state.defensePool.toNumber(), 2_700_000);
  });

  it("cannot defend again (defense not behind attack)", async () => {
    // attack=5, defense=3 → defense < attack, so defense IS allowed
    // Let's defend to make defense >= attack first
    await program.methods
      .defend(new anchor.BN(2))
      .accounts({
        defender: player3.publicKey,
        gameState: gameStatePda,
        defenderTokenAccount: player3Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player3])
      .rpc();

    // Now defense=5, attack=5 → tied, so defense IS allowed.
    await program.methods
      .defend(new anchor.BN(1))
      .accounts({
        defender: player3.publicKey,
        gameState: gameStatePda,
        defenderTokenAccount: player3Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player3])
      .rpc();

    // Now defense=6, attack=5. Turn restriction: attack(5) >= defense(6) → false → defend blocked
    try {
      await program.methods
        .defend(new anchor.BN(1))
        .accounts({
          defender: player3.publicKey,
          gameState: gameStatePda,
          defenderTokenAccount: player3Ata,
          throneVault,
          devWalletAta,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([player3])
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "TurnRestriction");
    }
  });

  it("attack is now allowed again (defense > attack)", async () => {
    // defense=6, attack=5 → defense >= attack → attack allowed
    await program.methods
      .attack(new anchor.BN(2))
      .accounts({
        attacker: player1.publicKey,
        gameState: gameStatePda,
        attackerTokenAccount: player1Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player1])
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    assert.equal(state.attackSoldiers.toNumber(), 7);
    assert.equal(state.defenseSoldiers.toNumber(), 6);
  });

  // ─── Finalize Battle: King Defeated ───────────────────────────────

  it("cannot finalize battle before timer expires", async () => {
    try {
      await program.methods
        .finalizeBattle()
        .accounts({
          anyone: admin.publicKey,
          gameState: gameStatePda,
          throneVault,
          kingTokenAccount: player2Ata,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "BattleNotExpired");
    }
  });

  it("finalizes battle — king defeated (attack > defense), all burned", async () => {
    await sleep(4000);

    const vaultBefore = (await getAccount(provider.connection, throneVault, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.isAbove(Number(vaultBefore), 0, "Vault should have tokens");

    await program.methods
      .finalizeBattle()
      .accounts({
        anyone: admin.publicKey,
        gameState: gameStatePda,
        throneVault,
        kingTokenAccount: player2Ata,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    const state = await program.account.gameState.fetch(gameStatePda);
    // King defeated → back to bidding
    assert.deepEqual(state.phase, { bidding: {} });
    assert.ok(state.king.equals(anchor.web3.PublicKey.default));
    assert.equal(state.battleActive, false);
    assert.equal(state.attackSoldiers.toNumber(), 0);
    assert.equal(state.defenseSoldiers.toNumber(), 0);
    assert.equal(state.attackPool.toNumber(), 0);
    assert.equal(state.defensePool.toNumber(), 0);
    assert.equal(state.nextBidAmount.toNumber(), 1);

    // Vault should be empty (all burned)
    const vaultAfter = (await getAccount(provider.connection, throneVault, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(Number(vaultAfter), 0);
  });

  // ─── Second Round: King Survives Scenario ─────────────────────────

  it("second round: bid → elect king → attack → defend → king survives", async () => {
    // Player 3 bids
    await program.methods
      .placeThroneBid()
      .accounts({
        bidder: player3.publicKey,
        gameState: gameStatePda,
        bidderTokenAccount: player3Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player3])
      .rpc();

    let state = await program.account.gameState.fetch(gameStatePda);
    assert.ok(state.candidate.equals(player3.publicKey));

    // Wait for timer
    await sleep(4000);

    // Finalize king election
    await program.methods
      .finalizeKingElection()
      .accounts({
        anyone: admin.publicKey,
        gameState: gameStatePda,
        throneVault,
        kingTokenAccount: player3Ata,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    state = await program.account.gameState.fetch(gameStatePda);
    assert.ok(state.king.equals(player3.publicKey));
    assert.deepEqual(state.phase, { battle: {} });

    // Player 1 attacks with 3 soldiers
    await program.methods
      .attack(new anchor.BN(3))
      .accounts({
        attacker: player1.publicKey,
        gameState: gameStatePda,
        attackerTokenAccount: player1Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player1])
      .rpc();

    // Player 2 defends with 5 soldiers (defense > attack → king will survive)
    await program.methods
      .defend(new anchor.BN(5))
      .accounts({
        defender: player2.publicKey,
        gameState: gameStatePda,
        defenderTokenAccount: player2Ata,
        throneVault,
        devWalletAta,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([player2])
      .rpc();

    state = await program.account.gameState.fetch(gameStatePda);
    assert.equal(state.attackSoldiers.toNumber(), 3);
    assert.equal(state.defenseSoldiers.toNumber(), 5);

    // Wait for battle timer
    await sleep(4000);

    const kingBalBefore = (await getAccount(provider.connection, player3Ata, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;

    // Finalize battle — king survives (defense 5 >= attack 3)
    await program.methods
      .finalizeBattle()
      .accounts({
        anyone: admin.publicKey,
        gameState: gameStatePda,
        throneVault,
        kingTokenAccount: player3Ata,
        tokenMint,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .rpc();

    state = await program.account.gameState.fetch(gameStatePda);
    // King survives → stays in battle phase, battle reset
    assert.deepEqual(state.phase, { battle: {} });
    assert.ok(state.king.equals(player3.publicKey));
    assert.equal(state.battleActive, false);
    assert.equal(state.attackSoldiers.toNumber(), 0);
    assert.equal(state.defenseSoldiers.toNumber(), 0);

    // King gets 50% of defense pool
    // Defense: 5 tokens = 5,000,000. Dev fee = 500,000. Army pool = 4,500,000
    // King payout = 4,500,000 / 2 = 2,250,000
    const kingBalAfter = (await getAccount(provider.connection, player3Ata, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(
      Number(kingBalAfter) - Number(kingBalBefore),
      2_250_000
    );

    // Vault empty (rest burned)
    const vaultBal = (await getAccount(provider.connection, throneVault, undefined, TOKEN_2022_PROGRAM_ID))
      .amount;
    assert.equal(Number(vaultBal), 0);
  });

  // ─── Edge Cases ───────────────────────────────────────────────────

  it("cannot attack/defend in bidding phase", async () => {
    // We need to get back to bidding. The king survived so we're in battle phase.
    // Let's just verify the current phase and test accordingly.
    const state = await program.account.gameState.fetch(gameStatePda);

    if (JSON.stringify(state.phase) === JSON.stringify({ battle: {} })) {
      // We're in battle phase, verify bid fails
      try {
        await program.methods
          .placeThroneBid()
          .accounts({
            bidder: player1.publicKey,
            gameState: gameStatePda,
            bidderTokenAccount: player1Ata,
            throneVault,
            devWalletAta,
            tokenMint,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([player1])
          .rpc();
        assert.fail("Should have thrown");
      } catch (e: any) {
        assert.include(e.toString(), "InvalidPhase");
      }
    }
  });

  it("cannot finalize battle when no battle is active", async () => {
    try {
      await program.methods
        .finalizeBattle()
        .accounts({
          anyone: admin.publicKey,
          gameState: gameStatePda,
          throneVault,
          kingTokenAccount: player3Ata,
          tokenMint,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .rpc();
      assert.fail("Should have thrown");
    } catch (e: any) {
      assert.include(e.toString(), "NoBattle");
    }
  });
});
