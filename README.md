# Update rust and cargo
rustup update


# Build release
cargo build --release

# Copy new binary
sudo cp target/release/eventsource /usr/local/bin/sensor-database-service

# Restart service
sudo systemctl start sensor-database.service
