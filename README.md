# RPC-providers

## Monitoring
Monitoring is done with grafana, prometheus endpoints in different geographical zones and alertmanager for alerting. The public monitor is reachable here: 
* [https://monitor.rpc-providers.net](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1)

You can choose between networks, monitoring zones and providers in the dashboard selector or preselect via link parameters, for example:
* [All polkadot endpoints from all monitoring zones](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1&var-zone=All&var-network=polkadot&var-wss=All)
* [All kusama endpoints from all monitoring zones](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1&var-zone=All&var-network=kusama&var-wss=All)
* [All polkadot endpoints from zone eu-central](https://monitor.rpc-providers.net/d/bdkaq43z8xybka/rpc-providers?orgId=1&var-zone=eu-central&var-network=polkadot&var-wss=All)

## Monitoring endpoints
There are multipele prometheus monitoring endpoints active:
* [http://mon-us-east.rpc-providers.net/](http://mon-us-east.rpc-providers.net/) (Virginia)
* [http://mon-eu-central.rpc-providers.net/](http://mon-eu-central.rpc-providers.net/) (Frankfurt)
* [http://mon-ap-southeast.rpc-providers.net/](http://mon-ap-southeast.rpc-providers.net/) (Singapore) 

At this moment the data is updated every 15 minutes. 

A request for an extra endpoint can be done by creating an [issue](https://github.com/rpc-providers/rpc-monitor/issues).

## Prometheus configuration
If you want to include endpoints in your own prometheus scraper here is an example configuration:

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

## WSS Endpoint configuration
The WSS endpoints are configured by a file `config-<zone>.sh`. So the configuration for zone `eu-central` can be found in [config-eu-central.sh](https://github.com/rpc-providers/rpc-monitor/blob/master/config-eu-central.sh). Providers are placed in principle in their primary zone but can be placed in multiple zones (for example when using geo steered load balancing). Updates can be requested by creating a pull request of creating an [issue](https://github.com/rpc-providers/rpc-monitor/issues).
