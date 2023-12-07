import json
from locust import HttpUser, task, between

CREATE_TX_PATH = "/transactions"

with open("txs.json") as f:
    transactions = json.load(f) or []

print("num txs", len(transactions))

class MyUser(HttpUser):
    wait_time = between(1, 2)

    @task
    def create_transaction(self):
        tx = transactions.pop(0)

        response = self.client.post(CREATE_TX_PATH, json=tx)
        assert response.ok
        job_id = response.json()["jobId"]

        while True:
            response = self.client.get(f"/job/{job_id}").json()
            state = response["state"]
            if state == "completed":
                break
            elif state == "in_progress":
                print("In progress")
            elif state == "pending":
                print("pending")
            elif state == "failed":
                raise Exception(response["error"])
            else:
                raise Exception("unknown job status")

        self.environment.events.request_success.fire(
            request_type="POST",
            name=CREATE_TX_PATH,
            response_time=response.elapsed.total_seconds() * 1000,
        )


# async def wait_job_completed(job_id):
#     INTERVAL_S = 0.2
#     while True:
#         async with aiohttp.ClientSession() as session:
#             async with session.get(RELAYER_URL + f'/job/{job_id}') as response:
#                 r = await response.json()
#                 if response.status != 200:
#                     raise Exception('Relayer is not available')
#                 if r['state'] == 'failed':
#                     raise Exception(
#                         f'Transaction[job {job_id}] failed with reason: {r["failedReason"]}')
#                 elif r['state'] == 'completed':
#                     return r['txHash']
#                 await asyncio.sleep(INTERVAL_S)
