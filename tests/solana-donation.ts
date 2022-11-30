import * as anchor from "@project-serum/anchor";
import { Program, web3 } from "@project-serum/anchor";
import { SolanaDonation } from "../target/types/solana_donation";
import chai, { assert, expect } from "chai";
import chaiAsPromised from "chai-as-promised";

chai.use(chaiAsPromised);

const { SystemProgram } = anchor.web3;


describe("solana-donation", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const program = anchor.workspace.SolanaDonation as Program<SolanaDonation>;
  const donatesData = program.account.donates;
  const donatorData = program.account.donator;

  const systemProgram = SystemProgram.programId;
  let owner = provider.wallet;
  let authority = owner.publicKey;

  async function find_donate_platform(authority: anchor.web3.PublicKey) {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("donate_platform"), authority.toBuffer()], program.programId
    );
  }

  async function find_donator_acc(donatePlatform: anchor.web3.PublicKey, id: number) {
    return await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("donate_platform_donator"), donatePlatform.toBuffer(), Buffer.from(id.toString())], program.programId
    );
  }

  async function get_lamports(to: anchor.web3.PublicKey) {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(to, 20 * anchor.web3.LAMPORTS_PER_SOL),
      "confirmed"
    );
  }

  async function get_balance(address: anchor.web3.PublicKey) {
    return await provider.connection.getBalance(address);
  }

  let donatorKeypair = anchor.web3.Keypair.generate();
  let donator = donatorKeypair.publicKey;

  before(async () => {
    await get_lamports(donator);
  });


  it("Program is initialized correctly", async () => {
    let [donatePlatform] = await find_donate_platform(authority);

    const target = 10000;
    await program.methods
      .initialize(new anchor.BN(target))
      .accounts({
        donatePlatform,
        authority,
      })
      .rpc();

    let donates = await donatesData.fetch(donatePlatform);
    assert.equal(
      donates.target, target,
      "Targets are not the same!"
    );
    assert.deepEqual(
      donates.authority, authority,
      "Authorities are not the same!"
    );
    assert.equal(
      donates.collected, 0,
      "Collected amount is not zero!"
    );
  });

  it("Random user can't withdraw lamports", async () => {
    let [donatePlatform] = await find_donate_platform(authority);
    expect((async () =>
        await program.methods
          .withdraw()
          .accounts({
            donatePlatform,
            authority: donator
          })
          .signers([donatorKeypair])
          .rpc()
    )()).to.be.rejectedWith(/A has one constraint was violated/);
  });

  it("Authority can withdraw lamports", async () => {
    let [donatePlatform] = await find_donate_platform(authority);
    let progBefore = await get_balance(donatePlatform);
    let authBefore = await get_balance(authority);
    let collected = (await program.account.donates.fetch(donatePlatform)).collected.toNumber();

    await program.methods
      .withdraw()
      .accounts({
        donatePlatform,
        authority
      })
      .rpc();

    let progAfter = await get_balance(donatePlatform);
    let authAfter = await get_balance(authority);

    assert.equal(
      progBefore - progAfter - collected, authAfter - authBefore,
      "Difference between collected authority and doesn't match!"
    );
  });

  it("Authority can't withdraw lamports if collected == 0", async () => {
    let [donatePlatform] = await find_donate_platform(authority);
    expect((async () =>
        await program.methods
          .withdraw()
          .accounts({
            donatePlatform,
            authority
          })
          .rpc()
    )()).to.be.rejectedWith(/A has one constraint was violated/);
  });

  
  


});
