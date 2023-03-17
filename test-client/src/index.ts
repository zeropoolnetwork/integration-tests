import { EthereumClient, Client as NetworkClient } from 'zeropool-support-js';
import { init, NearNetwork, ZeropoolClient } from 'zeropool-client-js';
import HDWalletProvider from '@truffle/hdwallet-provider';
import { deriveSpendingKey } from 'zeropool-client-js/lib/utils';
import { NetworkType } from 'zeropool-client-js/lib/network-type';
import { EvmNetwork } from 'zeropool-client-js/lib/networks/evm';

class Context {
  constructor(
    public zpClient: ZeropoolClient,
    public evmClient: EthereumClient,
    public tokenAddress: string,
    public poolAddress: string,
  ) { }

  static async create(rpcUrl: string, poolAddress: string, tokenAddress: string, relayerUrl: string, mnemonic: string): Promise<Context> {
    const snarkParamsConfig = {
      transferParamsUrl: './transfer_params.bin',
      treeParamsUrl: './tree_params.bin',
      transferVkUrl: './transfer_verification_key.json',
      treeVkUrl: './tree_verification_key.json',
    };

    console.log('Initializing worker...');
    const { worker, snarkParams } = await init(snarkParamsConfig);

    const provider = new HDWalletProvider({
      mnemonic,
      providerOrUrl: rpcUrl,
    });

    // @ts-ignore
    const evmClient = new EthereumClient(provider);
    const network = new EvmNetwork(rpcUrl);

    const sk = deriveSpendingKey(mnemonic, NetworkType.ethereum);

    console.log('Creating ZeropoolClient...')
    const zpClient = await ZeropoolClient.create({
      sk,
      worker,
      snarkParams,
      tokens: {
        [tokenAddress]: {
          poolAddress,
          relayerUrl,
        }
      },
      networkName: 'test',
      network,
    });

    return new Context(zpClient, evmClient, tokenAddress, poolAddress);
  }

  async deposit(amount: string): Promise<{ approveTime: number, txTime: number, fullTime: number }> {
    await this.evmClient.mint(this.tokenAddress, amount);

    const [, approveTime] = await measureTime(async () => {
      await this.evmClient.approve(this.tokenAddress, this.poolAddress, amount);
    });

    const [jobId, txTime] = await measureTime(async () => {
      return await this.zpClient.deposit(this.tokenAddress, BigInt(amount), (data) => this.evmClient.sign(data), null, BigInt(0), []);
    });

    const [, fullTime] = await measureTime(async () => {
      await this.zpClient.waitJobCompleted(this.tokenAddress, jobId);
    });

    return {
      approveTime,
      txTime,
      fullTime,
    }
  }

  async transfer(amount: string, to: string): Promise<{ txTime: number, fullTime: number }> {
    const [jobId, txTime] = await measureTime(async () => {
      return await this.zpClient.transfer(this.tokenAddress, [{
        to,
        amount,
      }]);
    });

    const [, fullTime] = await measureTime(async () => {
      await this.zpClient.waitJobCompleted(this.tokenAddress, jobId);
    });

    return {
      txTime,
      fullTime,
    }
  }

  async withdraw(amount: string, to: string): Promise<{ txTime: number, fullTime: number }> {
    const [jobId, txTime] = await measureTime(async () => {
      return await this.zpClient.withdraw(this.tokenAddress, to, BigInt(amount));
    });

    const [, fullTime] = await measureTime(async () => {
      await this.zpClient.waitJobCompleted(this.tokenAddress, jobId);
    });

    return {
      txTime,
      fullTime,
    }
  }
}

async function measureTime<T>(fn: () => Promise<T>): Promise<[T, number]> {
  const start = Date.now();
  const res = await fn();
  const end = Date.now();

  return [res, end - start];
}


global.start = async function start(rpcUrl: string, poolAddress: string, tokenAddress: string, relayerUrl: string, mnemonic: string) {
  const ctx = await Context.create(rpcUrl, poolAddress, tokenAddress, relayerUrl, mnemonic);

  // const publicAddress = ctx.evmClient.getAddress();
  const shieldedAddress = ctx.zpClient.generateAddress(tokenAddress);

  console.log('Shielded address generated', shieldedAddress);

  // Deposit 3 eth
  const depositTimes = ctx.deposit('3000000000000000000');
  const shieldedBalanceAfterDeposit = await ctx.zpClient.getOptimisticTotalBalance(tokenAddress, true);
  const publicBalanceAfterDeposit = await ctx.evmClient.getTokenBalance(tokenAddress);

  console.log('Deposit done');

  // Transfer 1 eth to self
  const transferTimes = ctx.transfer('1000000000000000000', shieldedAddress);
  const shieldedBalanceAfterTransfer = await ctx.zpClient.getOptimisticTotalBalance(tokenAddress, true);
  const publicBalanceAfterTransfer = await ctx.evmClient.getTokenBalance(tokenAddress);

  console.log('Transfer done');

  // Should be able to withdraw all 3 eth
  const withdrawTimes = ctx.withdraw('3000000000000000000', shieldedAddress);
  const shieldedBalanceAfterWithdraw = await ctx.zpClient.getOptimisticTotalBalance(tokenAddress, true);
  const publicBalanceAfterWithdraw = await ctx.evmClient.getTokenBalance(tokenAddress);

  console.log('Withdraw done');

  return {
    depositTimes,
    transferTimes,
    withdrawTimes,
    shieldedBalanceAfterDeposit,
    publicBalanceAfterDeposit,
    shieldedBalanceAfterTransfer,
    publicBalanceAfterTransfer,
    shieldedBalanceAfterWithdraw,
    publicBalanceAfterWithdraw,
  };
}
