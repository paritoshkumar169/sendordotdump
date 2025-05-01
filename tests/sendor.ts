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
} from "@solana/spl-token";

describe("sendor - full launch test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.sendor as Program<any>;
  const connection = provider.connection; // ✅ USE THIS ONLY

  const adminKeypair = Keypair.generate(); // ✅ Using REAL Keypair for admin
  const users = [Keypair.generate(), Keypair.generate(), Keypair.generate()];

  let globalStatePda: PublicKey;
  let launchMetadataPda: PublicKey;
  let bondingCurvePda: PublicKey;
  let tokenMint: PublicKey;
  let vault: PublicKey;

  it("Airdrops SOL to admin", async () => {
    await connection.requestAirdrop(adminKeypair.publicKey, LAMPORTS_PER_SOL * 5);
    await new Promise(resolve => setTimeout(resolve, 3000));
  });

  it("Initializes global state", async () => {
    [globalStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("global")],
      program.programId
    );

    await program.rpc.initialize({
      accounts: {
        globalState: globalStatePda,
        admin: adminKeypair.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [adminKeypair],
    });
  });

  it("Creates a new token launch", async () => {
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
      connection as any,
      adminKeypair, // ✅ real Keypair
      launchMetadataPda,
      null,
      9
    );

    const vaultAccount = await getOrCreateAssociatedTokenAccount(
      connection as any,
      adminKeypair, // ✅ real Keypair
      tokenMint,
      launchMetadataPda,
      true
    );
    vault = vaultAccount.address;

    await program.rpc.createLaunch(
      new anchor.BN(1000),
      new anchor.BN(1),
      {
        accounts: {
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
        },
        signers: [adminKeypair],
      }
    );
  });

  it("Deployer buys token (starts bonding curve)", async () => {
    const adminAta = await getOrCreateAssociatedTokenAccount(
      connection as any,
      adminKeypair,
      tokenMint,
      adminKeypair.publicKey
    );

    await program.rpc.buy(
      new anchor.BN(1_000_000_000),
      {
        accounts: {
          launchMetadata: launchMetadataPda,
          bondingCurve: bondingCurvePda,
          tokenMint,
          vault,
          buyer: adminKeypair.publicKey,
          buyerTokenAccount: adminAta.address,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [adminKeypair],
      }
    );
  });

  it("Users buy token", async () => {
    for (const user of users) {
      await connection.requestAirdrop(user.publicKey, LAMPORTS_PER_SOL * 2);
      await new Promise(resolve => setTimeout(resolve, 3000));

      const userAta = await getOrCreateAssociatedTokenAccount(
        connection as any,
        user,
        tokenMint,
        user.publicKey
      );

      await program.rpc.buy(
        new anchor.BN(1_000_000_000),
        {
          accounts: {
            launchMetadata: launchMetadataPda,
            bondingCurve: bondingCurvePda,
            tokenMint,
            vault,
            buyer: user.publicKey,
            buyerTokenAccount: userAta.address,
            tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
            associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
          signers: [user],
        }
      );
    }
  });
});
