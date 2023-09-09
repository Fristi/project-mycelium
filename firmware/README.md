## firmware

Built with Rust and ESP IDF (Bluedroid, WiFi and HTTP client)

### Flash without erasing settings

```
MYCELIUM_BASE_URL=https://mycelium.fly.dev cargo +esp espflash flash --baud 2000000  --target xtensa-esp32-espidf --release
```

### Flash with erasing settings

```
MYCELIUM_BASE_URL=https://mycelium.fly.dev cargo +esp espflash flash --erase-parts nvs --baud 2000000  --target xtensa-esp32-espidf --release
```