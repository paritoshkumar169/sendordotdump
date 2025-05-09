import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  getMint,
  getAccount,
} from "@solana/spl-token";
import { Sendor } from "../target/types/sendor";

describe("sendor - initialization and launch tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.sendor as Program<Sendor>;
  const connection = provider.connection;

  const adminKeypair = Keypair.generate();
  const unauthorizedUser = Keypair.generate();
  
  let globalStatePda: PublicKey;
  let launchMetadataPda: PublicKey;
  let bondingCurvePda: PublicKey;
  let tokenMint: PublicKey;
  let vault: PublicKey;

  before(async () => {
    // Setup: Airdrop SOL to admin and unauthorized user
    await Promise.all([
      connection.requestAirdrop(adminKeypair.publicKey, LAMPORTS_PER_SOL * 5),
      connection.requestAirdrop(unauthorizedUser.publicKey, LAMPORTS_PER_SOL * 2)
    ]);
    await new Promise(resolve => setTimeout(resolve, 3000));
  });

  describe("Initialize Global State", () => {
    it("should initialize global state with correct admin", async () => {
      [globalStatePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("global")],
        program.programId
      );

      await program.methods
        .initialize()
        .accounts({
          globalState: globalStatePda,
          admin: adminKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([adminKeypair])
        .rpc();

      // Verify the global state
      const globalState = await program.account.globalState.fetch(globalStatePda);
      assert.ok(globalState.admin.equals(adminKeypair.publicKey), "Admin not set correctly");
      assert.equal(globalState.launchCount.toNumber(), 0, "Launch count should be 0");
    });

    it("should fail initializing global state again", async () => {
      try {
        await program.methods
          .initialize()
          .accounts({
            globalState: globalStatePda,
            admin: adminKeypair.publicKey,
            systemProgram: SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([adminKeypair])
          .rpc();
        assert.fail("Should not allow second initialization");
      } catch (error) {
        assert.include(error.message, "Error Code: AccountAlreadyInitialized");
      }
    });
  });

  describe("Create Launch", () => {
    const basePrice = 1; // 1 lamport
    const slope = 1;
    
    it("should create a new token launch successfully", async () => {
      const seed = new anchor.BN(0).toArrayLike(Buffer, "le", 8);

      [launchMetadataPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("launch"), seed],
        program.programId
      );
      [bondingCurvePda] = PublicKey.findProgramAddressSync(
        [Buffer.from("bonding"), seed],
        program.programId
      );

      tokenMint = await createMint(
        connection,
        adminKeypair,
        adminKeypair.publicKey, // Change mint authority to adminKeypair initially
        launchMetadataPda,     // Set freeze authority to launch metadata PDA
        9 // TOKEN_DECIMALS
      );

      const vaultAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        adminKeypair,
        tokenMint,
        launchMetadataPda,
        true
      );
      vault = vaultAccount.address;

      await program.methods
        .createLaunch(
          new anchor.BN(basePrice),
          new anchor.BN(slope)
        )
        .accounts({
          globalState: globalStatePda,
          launchMetadata: launchMetadataPda,
          bondingCurve: bondingCurvePda,
          tokenMint,
          vault,
          admin: adminKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([adminKeypair])
        .rpc();

      // Verify launch metadata
      const launchMetadata = await program.account.launchMetadata.fetch(launchMetadataPda);
      assert.ok(launchMetadata.tokenMint.equals(tokenMint), "Token mint not set correctly");
      assert.ok(launchMetadata.vault.equals(vault), "Vault not set correctly");
      assert.equal(launchMetadata.launchId.toNumber(), 0, "Launch ID should be 0");
      assert.equal(launchMetadata.currentDay.toNumber(), 0, "Current day should be 0");

      // Verify bonding curve state
      const bondingCurve = await program.account.bondingCurveState.fetch(bondingCurvePda);
      assert.ok(bondingCurve.launchMetadata.equals(launchMetadataPda), "Launch metadata not set correctly");
      assert.equal(bondingCurve.basePrice.toNumber(), basePrice, "Base price not set correctly");
      assert.equal(bondingCurve.slope.toNumber(), slope, "Slope not set correctly");
      assert.equal(bondingCurve.currentSupply.toNumber(), 0, "Current supply should be 0");
      assert.equal(bondingCurve.decimals, 9, "Decimals should be 9");

      // Verify token mint authority
      const mintInfo = await getMint(connection, tokenMint);
      assert.ok(mintInfo.mintAuthority === null, "Mint authority should be disabled");
      assert.ok(mintInfo.freezeAuthority?.equals(launchMetadataPda), "Freeze authority not set correctly");

      // Verify vault
      const vaultInfo = await getAccount(connection, vault);
      assert.ok(vaultInfo.amount.toString() === "1000000000000000000", "Initial supply not minted to vault");
      assert.ok(vaultInfo.owner.equals(launchMetadataPda), "Vault owner not set correctly");
    });

    it("should fail when unauthorized user tries to create launch", async () => {
      const seed = new anchor.BN(1).toArrayLike(Buffer, "le", 8);
      const [newLaunchPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("launch"), seed],
        program.programId
      );
      const [newBondingPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("bonding"), seed],
        program.programId
      );

      const newMint = await createMint(
        connection,
        unauthorizedUser,
        newLaunchPda,
        null,
        9
      );

      const newVault = await getOrCreateAssociatedTokenAccount(
        connection,
        unauthorizedUser,
        newMint,
        newLaunchPda,
        true
      );

      try {
        await program.methods
          .createLaunch(
            new anchor.BN(basePrice), 
            new anchor.BN(slope)
          )
          .accounts({
            globalState: globalStatePda,
            launchMetadata: newLaunchPda,
            bondingCurve: newBondingPda,
            tokenMint: newMint,
            vault: newVault.address,
            admin: unauthorizedUser.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([unauthorizedUser])
          .rpc();
        assert.fail("Should not allow unauthorized user to create launch");
      } catch (error) {
        assert.include(error.message, "Constraint: Has One Failed");
      }
    });

    it("should fail when invalid parameters are provided", async () => {
      try {
        await program.methods
          .createLaunch(
            new anchor.BN(0), 
            new anchor.BN(slope)
          ) // Invalid base price
          .accounts({
            globalState: globalStatePda,
            launchMetadata: launchMetadataPda,
            bondingCurve: bondingCurvePda,
            tokenMint,
            vault,
            admin: adminKeypair.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([adminKeypair])
          .rpc();
        assert.fail("Should not allow invalid base price");
      } catch (error) {
        assert.include(error.message, "InvalidParams");
      }
    });
  });
});
