import os
from selenium import webdriver

driver = webdriver.Chrome()
driver.get(os.environ['CLIENT_URL'])

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

print(f"Received message: {res}")
