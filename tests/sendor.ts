import * as anchor from '@project-serum/anchor'
import { Program, BN } from '@project-serum/anchor'
import { SendorDump } from '../target/types/sendor_dump'
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
  getAssociatedTokenAddress,
} from '@solana/spl-token'
import { assert, expect } from 'chai'
import { Keypair, PublicKey, SystemProgram, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js'
import { airdrop } from './utils/airdrop'

describe('sendor-dump', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const program = anchor.workspace.SendorDump as Program<SendorDump>

  // Constants from Rust program
  const INITIAL_SUPPLY = new BN(1_000_000_000)
  const DECIMALS = 9
  const SELL_WINDOW_DURATION = 15 * 60 // 15 minutes
  const DAY_DURATION = 24 * 60 * 60 // 24 hours

  // Accounts
  const admin = provider.wallet
  let globalState: PublicKey
  let launchCount = 0
  let tokenMint: PublicKey
  let launchMetadata: PublicKey
  let bondingCurveState: PublicKey
  let user = Keypair.generate()
  let userTokenAccount: PublicKey

  before(async () => {
    // Airdrop to user
    await airdrop(provider.connection, user.publicKey, 100 * anchor.web3.LAMPORTS_PER_SOL)

    // Derive global state PDA
    ;[globalState] = await PublicKey.findProgramAddress(
      [Buffer.from('global-config')],
      program.programId
    )
  })

  describe('Initialize', () => {
    it('Successfully initializes global state', async () => {
      await program.methods.initialize().accounts({ globalState }).rpc()
      const state = await program.account.globalState.fetch(globalState)
      assert.equal(state.admin.toString(), admin.publicKey.toString())
      assert.equal(state.launchCount.toNumber(), 0)
    })

    it('Fails if already initialized', async () => {
      try {
        await program.methods.initialize().accounts({ globalState }).rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'Allocate: account Address { address:')
      }
    })
  })

  describe('Create Launch', () => {
    it('Admin can create new token launch', async () => {
      const [launch] = await PublicKey.findProgramAddress(
        [Buffer.from('launch-metadata'), new BN(launchCount).toArrayLike(Buffer, 'le', 8)],
        program.programId
      )

      const tx = await program.methods
        .createLaunch(new BN(100), new BN(10))
        .accounts({ globalState, admin: admin.publicKey })
        .rpc()

      // Fetch created accounts
      launchMetadata = launch
      const metadata = await program.account.launchMetadata.fetch(launch)
      tokenMint = metadata.tokenMint
      bondingCurveState = metadata.bondingCurveState

      // Verify bonding curve params
      const curve = await program.account.bondingCurveState.fetch(bondingCurveState)
      assert.equal(curve.basePrice.toNumber(), 100)
      assert.equal(curve.slope.toNumber(), 10)

      // Verify mint creation
      const mintInfo = await provider.connection.getAccountInfo(tokenMint)
      assert.ok(mintInfo)

      // Verify supply
      const vault = await getAssociatedTokenAddress(tokenMint, launchMetadata, true)
      const vaultInfo = await getAccount(provider.connection, vault)
      assert.equal(vaultInfo.amount.toString(), INITIAL_SUPPLY.toString())
    })

    it('Non-admin cannot create launch', async () => {
      const hacker = Keypair.generate()
      await airdrop(provider.connection, hacker.publicKey, 1)

      try {
        await program.methods
          .createLaunch(new BN(100), new BN(10))
          .accounts({
            globalState,
            admin: hacker.publicKey,
          })
          .signers([hacker])
          .rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'Unauthorized')
      }
    })
  })

  describe('Buy Tokens', () => {
    before(async () => {
      // Create user token account
      userTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        user,
        tokenMint,
        user.publicKey
      )
    })

    it('Successfully buys tokens', async () => {
      const amount = new BN(1_000_000)
      const curve = await program.account.bondingCurveState.fetch(bondingCurveState)
      const expectedPrice = curve.basePrice.add(curve.slope.mul(curve.currentSupply))

      await program.methods
        .buy(amount)
        .accounts({
          buyer: user.publicKey,
          tokenAccount: userTokenAccount,
          launchMetadata,
          bondingCurveState,
        })
        .signers([user])
        .rpc()

      // Check user balance
      const userBalance = await getAccount(provider.connection, userTokenAccount)
      assert.equal(userBalance.amount.toString(), amount.toString())

      // Check updated bonding curve
      const updatedCurve = await program.account.bondingCurveState.fetch(bondingCurveState)
      assert.equal(updatedCurve.currentSupply.toString(), amount.toString())
    })

    it('Fails with insufficient SOL', async () => {
      const largeAmount = new BN(INITIAL_SUPPLY)
      try {
        await program.methods
          .buy(largeAmount)
          .accounts({
            buyer: user.publicKey,
            tokenAccount: userTokenAccount,
            launchMetadata,
            bondingCurveState,
          })
          .signers([user])
          .rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'InsufficientFunds')
      }
    })
  })

  describe('Sell Tokens', () => {
    let userRecord: PublicKey

    before(async () => {
      // Derive user record PDA
      ;[userRecord] = await PublicKey.findProgramAddress(
        [Buffer.from('user_record'), launchMetadata.toBuffer(), user.publicKey.toBuffer()],
        program.programId
      )

      // Move time into sell window
      await program.methods
        .updateGlobal()
        .accounts({ globalState, admin: admin.publicKey })
        .rpc()
    })

    it('Successfully sells tokens within window', async () => {
      const sellAmount = new BN(100_000)
      const preBalance = await provider.connection.getBalance(user.publicKey)

      await program.methods
        .sell(sellAmount)
        .accounts({
          user: user.publicKey,
          tokenAccount: userTokenAccount,
          launchMetadata,
          bondingCurveState,
          userRecord,
        })
        .signers([user])
        .rpc()

      // Check SOL received
      const postBalance = await provider.connection.getBalance(user.publicKey)
      assert.isAbove(postBalance, preBalance)

      // Check token balance
      const userBalance = await getAccount(provider.connection, userTokenAccount)
      assert.equal(userBalance.amount.toString(), new BN(900_000).toString())
    })

    it('Fails outside sell window', async () => {
      // Advance time beyond window
      const metadata = await program.account.launchMetadata.fetch(launchMetadata)
      const newTime = metadata.window1Start.add(new BN(DAY_DURATION))
      await program.methods.setSellWindow(newTime, newTime.add(new BN(SELL_WINDOW_DURATION))).rpc()

      try {
        await program.methods
          .sell(new BN(1))
          .accounts({ /* ... */ })
          .rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'NotInTradingWindow')
      }
    })

    it('Fails if exceeding daily limit', async () => {
      const excessAmount = new BN(200_000) // 20% of 1M initial
      try {
        await program.methods
          .sell(excessAmount)
          .accounts({ /* ... */ })
          .rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'ExceedsSellLimit')
      }
    })
  })

  describe('Transfer Tokens', () => {
    let receiver = Keypair.generate()
    let receiverAccount: PublicKey

    before(async () => {
      receiverAccount = await createAssociatedTokenAccount(
        provider.connection,
        user,
        tokenMint,
        receiver.publicKey
      )
    })

    it('Successfully transfers within limits', async () => {
      const transferAmount = new BN(50_000)
      await program.methods
        .transfer(transferAmount)
        .accounts({
          from: user.publicKey,
          to: receiver.publicKey,
          fromAccount: userTokenAccount,
          toAccount: receiverAccount,
          launchMetadata,
        })
        .signers([user])
        .rpc()

      const receiverBalance = await getAccount(provider.connection, receiverAccount)
      assert.equal(receiverBalance.amount.toString(), transferAmount.toString())
    })

    it('Fails if exceeding transfer limit', async () => {
      const excessAmount = new BN(300_000) // 30% of 1M
      try {
        await program.methods
          .transfer(excessAmount)
          .accounts({ /* ... */ })
          .rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'ExceedsTransferLimit')
      }
    })
  })

  describe('Admin Operations', () => {
    it('Can update global state', async () => {
      await program.methods
        .updateGlobal()
        .accounts({ globalState, admin: admin.publicKey })
        .rpc()

      const metadata = await program.account.launchMetadata.fetch(launchMetadata)
      assert.equal(metadata.currentDay.toNumber(), 1)
      assert.isAtLeast(metadata.window1Start.toNumber(), 0)
    })

    it('Non-admin cannot update', async () => {
      try {
        await program.methods
          .updateGlobal()
          .accounts({ globalState, admin: user.publicKey })
          .signers([user])
          .rpc()
        assert.fail('Should have thrown error')
      } catch (err) {
        assert.include(err.message, 'Unauthorized')
      }
    })
  })

  describe('Migration', () => {
    it('Admin can migrate launch', async () => {
      const adminTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        admin.payer,
        tokenMint,
        admin.publicKey
      )

      await program.methods
        .migrate()
        .accounts({
          admin: admin.publicKey,
          launchMetadata,
          tokenMint,
          adminTokenAccount,
        })
        .rpc()

      // Verify tokens transferred
      const adminBalance = await getAccount(provider.connection, adminTokenAccount)
      assert.equal(adminBalance.amount.toString(), INITIAL_SUPPLY.sub(new BN(900_000)).toString())
    })
  })
})