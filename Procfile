release: npm install --global yarn; cd ui; yarn build
web: ASYNC_STD_THREAD_COUNT=5 RUST_LOG=info HOST=0.0.0.0 PORT=$PORT ./target/release/flatterer-web
