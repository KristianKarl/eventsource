# Consume OpenHAP2 events

Populate a database given selected events of interests


## rust housdhold keeping

Update rust and cargo
```
rustup update
```

Build a release target
```
cargo build --release
```

## Deploy to production

Stop service and copy new binary

```
sudo systemctl stop sensor-database.service
sudo cp target/release/eventsource /usr/local/bin/sensor-database-service
```

Restart service
```
sudo systemctl start sensor-database.service

# Check status
sudo systemctl status sensor-database.service
```
