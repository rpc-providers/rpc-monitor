# rpc-providers github page

# Monitoring
Monitoring is done with grafana and reachable through this [website](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1).

For example:
* [All polkadot endpoints from all monitoring zones](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1&var-zone=All&var-network=polkadot&var-wss=All)
* [All kusama endpoints from all monitoring zones](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1&var-zone=All&var-network=kusama&var-wss=All)

# Monitoring endpoints
There are multipele prometheus monitoring endpoints:
* [http://mon-us-east.rpc-providers.net/](http://mon-us-east.rpc-providers.net/) (Virginia)
* [http://mon-eu-central.rpc-providers.net/](http://mon-eu-central.rpc-providers.net/) (Frankfurt)
* [http://mon-ap-southeast.rpc-providers.net/](http://mon-ap-southeast.rpc-providers.net/) (Singapore) 

# An example prometheus config could be:

```
global:
  scrape_interval: 10m
  evaluation_interval: 10m

scrape_configs:
  - job_name: "rpc-mon"
    metrics_path: "/"
    static_configs:
      - targets: ["mon-eu-central.rpc-providers.net:80"]
      - targets: ["mon-us-east.rpc-providers.net:80"]
      - targets: ["mon-ap-southeast.rpc-providers.net:80"]
```
