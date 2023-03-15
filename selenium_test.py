import os
from selenium import webdriver
from dotenv import load_dotenv

load_dotenv()

driver = webdriver.Chrome()
driver.get(os.environ['CLIENT_URL'] or "http://localhost:3000")

RPC_URL = os.environ['RPC_URL']
POOL_ADDRESS = os.environ['POOL_ADDRESS']
TOKEN_ADDRESS = os.environ['TOKEN_ADDRESS']
RELAYER_URL = os.environ['RELAYER_URL']
MNEMONIC = os.environ['MNEMONIC']

"""
Returns:
    depositTimes,
    transferTimes,
    withdrawTimes,
    shieldedBalanceAfterDeposit,
    publicBalanceAfterDeposit,
    shieldedBalanceAfterTransfer,
    publicBalanceAfterTransfer,
    shieldedBalanceAfterWithdraw,
    publicBalanceAfterWithdraw,
"""
res = driver.execute_async_script(
    "const callback = arguments[arguments.length - 1];"
    "const result = await window.start();"
    "callback(result);",
    RPC_URL,
    POOL_ADDRESS,
    TOKEN_ADDRESS,
    RELAYER_URL,
    MNEMONIC,
)

print("Results:")
print("\nDeposit Times:")
print(f"  Approve Time: {res['depositTimes']['approveTime']} ms")
print(f"  Tx Time: {res['depositTimes']['txTime']} ms")
print(f"  Confirmation Time: {res['depositTimes']['fullTime']} ms")

print("\nTransfer Times:")
print(f"  Tx Time: {res['transferTimes']['txTime']} ms")
print(f"  Confirmation Time: {res['transferTimes']['fullTime']} ms")

print("\nWithdraw Times:")
print(f"  Tx Time: {res['withdrawTimes']['txTime']} ms")
print(f"  Confirmation Time: {res['withdrawTimes']['fullTime']} ms")
