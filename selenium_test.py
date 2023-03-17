import os

from selenium import webdriver
from selenium.webdriver.common.desired_capabilities import DesiredCapabilities
from dotenv import load_dotenv

load_dotenv()

capabilities = DesiredCapabilities.CHROME
capabilities["goog:loggingPrefs"] = {"browser": "ALL"}
options = webdriver.ChromeOptions()
# options.add_argument("--headless")
options.add_argument("--disable-web-security")
options.add_argument("--allow-running-insecure-content")
options.add_argument("--enable-features=SharedArrayBuffer")
driver = webdriver.Chrome(options=options, desired_capabilities=capabilities)

driver.set_script_timeout(24 * 60 * 60)

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
"""
res = driver.execute_async_script(
    "const callback = arguments[arguments.length - 1];"
    "try {"
    "  const result = await window.start(...arguments);"
    "  callback(result);"
    "} catch (e) {"
    "  callback(e);"
    "}",
    RPC_URL,
    POOL_ADDRESS,
    TOKEN_ADDRESS,
    RELAYER_URL,
    MNEMONIC,
)

print("res:", res)

logs = driver.get_log("browser")
print("Browser logs:")
for log in logs:
    print(log)

# Close the WebDriver
driver.quit()

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
