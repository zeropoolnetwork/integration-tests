import os
import dotenv
import json
import asyncio
import aiohttp
import time
from web3 import Web3
# from web3.auto import w3
# from web3 import Account

dotenv.load_dotenv()

RELAYER_URL = os.environ['RELAYER_URL']
RPC_URL = os.environ['RPC_URL']
POOL_ADDRESS = os.environ['POOL_ADDRESS']
TOKEN_ADDRESS = os.environ['TOKEN_ADDRESS']
MNEMONIC = os.environ['MNEMONIC']

# TODO: mint tokens in the hardhat config
print('Minting tokens...')
web3 = Web3(Web3.HTTPProvider(RPC_URL))
web3.eth.account.enable_unaudited_hdwallet_features()
num_accounts = 10
accounts = [web3.eth.account.from_mnemonic(
    MNEMONIC, account_path=f"m/44'/60'/0'/0/{i}") for i in range(num_accounts)]
contract_abi = json.load(open('tests/token-abi.json'))

contract = web3.eth.contract(address=TOKEN_ADDRESS, abi=contract_abi)
token_decimals = contract.functions.decimals().call()
token_amount = 1000

for account in accounts:
    tx_hash = contract.functions.mint(
        account.address, token_amount * 10 ** token_decimals).transact({'from': account.address})
    tx_hash = contract.functions.approve(
        POOL_ADDRESS, token_amount * 10 ** token_decimals).transact({'from': account.address})
    tx_receipt = web3.eth.wait_for_transaction_receipt(tx_hash)
    print(f"Minted and approved {token_amount} to {account.address}")


async def wait_job_completed(job_id):
    INTERVAL_S = 0.2
    while True:
        async with aiohttp.ClientSession() as session:
            async with session.get(RELAYER_URL + f'/job/{job_id}') as response:
                r = await response.json()
                if response.status != 200:
                    raise Exception('Relayer is not available')
                if r['state'] == 'failed':
                    raise Exception(
                        f'Transaction[job {job_id}] failed with reason: {r["failedReason"]}')
                elif r['state'] == 'completed':
                    return r['txHash']
                await asyncio.sleep(INTERVAL_S)


async def main():
    results = []

    txs = []
    with open('txs.json', 'r') as file:
        txs = json.load(file)

    async with aiohttp.ClientSession() as session:
        async with session.get(RELAYER_URL + '/info') as response:
            r = await response.json()
            if response.status != 200:
                raise Exception('Relayer is not available')

        async def post_transaction(tx):
            print(tx)
            start = time.time()
            async with session.post(RELAYER_URL + '/sendTransactions', json=[tx]) as response:
                r = await response.json()
                print(response, r)

                if response.status != 200:
                    results.append({"error": r})
                    return

                job_id = r['jobId']

                tx_hash = await wait_job_completed(job_id)
                results.append({
                    "time": time.time() - start,
                })

        await asyncio.gather(*[post_transaction(tx) for tx in txs])

    # Pretty-print the results
    print(json.dumps(results, indent=2))


asyncio.run(main())
