#!/bin/zsh
cargo build --release
pm2 restart joycycles-backend