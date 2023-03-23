import os
import influxdb_client
from influxdb_client.client.write_api import SYNCHRONOUS


def submit():
    bucket = "performance"
    org = "main"
    token = os.environ['INFLUX_API_TOKEN']
    url = "https://grafana.zeropool.network"

    client = influxdb_client.InfluxDBClient(
        url=url,
        token=token,
        org=org
    )

    write_api = client.write_api(write_options=SYNCHRONOUS)

    p = influxdb_client.Point("test").tag(
        "location", "Prague").field("temperature", 25.3)
    write_api.write(bucket=bucket, org=org, record=p)
